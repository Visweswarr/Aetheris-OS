package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/spf13/cobra"
)

// BLE device information
type BleDeviceInfo struct {
	Address        string   `json:"address"`
	Name           string   `json:"name"`
	RSSI           int16    `json:"rssi"`
	ServiceUUIDs   []string `json:"service_uuids"`
	LastSeen       string   `json:"last_seen"`
}

// BLE scan filters
type BleScanFilters struct {
	NamePrefix     string   `json:"name_prefix,omitempty"`
	ServiceUUIDs   []string `json:"service_uuids,omitempty"`
	RSSIThreshold  int16    `json:"rssi_threshold,omitempty"`
	TimeoutMs      uint32   `json:"timeout_ms"`
}

// BLE notification data
type BleNotification struct {
	NotifyHandle   string    `json:"notify_handle"`
	ServiceUUID    string    `json:"service_uuid"`
	CharUUID       string    `json:"char_uuid"`
	Data           []byte    `json:"data"`
	Timestamp      time.Time `json:"timestamp"`
}

// Mock BLE service for demonstration
type MockBleService struct {
	activeScans        map[string]BleScan
	activeConnections  map[string]BleConnection
	discoveredDevices  []BleDeviceInfo
	activeNotifications map[string]BleNotificationStream
}

type BleScan struct {
	ScanHandle   string
	SessionID    string
	Filters      BleScanFilters
	StartTime    time.Time
	DevicesFound []BleDeviceInfo
}

type BleConnection struct {
	ConnHandle   string
	Address      string
	ConnectedAt  time.Time
	Services     []BleService
}

type BleService struct {
	UUID            string
	Characteristics []BleCharacteristic
}

type BleCharacteristic struct {
	UUID        string
	Properties  BleCharProperties
	Value       []byte
}

type BleCharProperties struct {
	Read      bool
	Write     bool
	Notify    bool
	Indicate  bool
}

type BleNotificationStream struct {
	NotifyHandle      string
	ConnHandle        string
	ServiceUUID       string
	CharUUID          string
	StartTime         time.Time
	NotificationCount uint32
}

var bleService = &MockBleService{
	activeScans:         make(map[string]BleScan),
	activeConnections:   make(map[string]BleConnection),
	discoveredDevices:   make([]BleDeviceInfo, 0),
	activeNotifications: make(map[string]BleNotificationStream),
}

// BLE commands
var bleCmd = &cobra.Command{
	Use:   "ble",
	Short: "Manage BLE device operations",
	Long:  "Manage Bluetooth Low Energy device operations, including scanning, connecting, and GATT operations.",
}

var bleScanCmd = &cobra.Command{
	Use:   "scan",
	Short: "Start BLE device scan",
	Long:  "Start scanning for BLE devices with optional filters.",
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		namePrefix, _ := cmd.Flags().GetString("name-prefix")
		serviceUUIDs, _ := cmd.Flags().GetString("service-uuids")
		rssiThreshold, _ := cmd.Flags().GetInt16("rssi-threshold")
		timeout, _ := cmd.Flags().GetUint32("timeout")
		jsonOutput, _ := cmd.Flags().GetBool("json")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:ble.scan"
		}

		filters := BleScanFilters{
			NamePrefix:    namePrefix,
			ServiceUUIDs:  strings.Split(serviceUUIDs, ","),
			RSSIThreshold: rssiThreshold,
			TimeoutMs:     timeout,
		}

		// Remove empty service UUIDs
		var validUUIDs []string
		for _, uuid := range filters.ServiceUUIDs {
			if strings.TrimSpace(uuid) != "" {
				validUUIDs = append(validUUIDs, strings.TrimSpace(uuid))
			}
		}
		filters.ServiceUUIDs = validUUIDs

		scanHandle, err := bleService.StartScan(sessionID, caps, filters)
		if err != nil {
			fmt.Printf("Error starting BLE scan: %v\n", err)
			return
		}

		if jsonOutput {
			result := map[string]interface{}{
				"scan_handle": scanHandle,
				"session_id":  sessionID,
				"filters":     filters,
				"success":     true,
			}
			jsonData, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			fmt.Printf("BLE scan started successfully\n")
			fmt.Printf("Scan handle: %s\n", scanHandle)
			fmt.Printf("Session ID: %s\n", sessionID)
			fmt.Printf("Timeout: %d ms\n", filters.TimeoutMs)
			if filters.NamePrefix != "" {
				fmt.Printf("Name prefix filter: %s\n", filters.NamePrefix)
			}
			if len(filters.ServiceUUIDs) > 0 {
				fmt.Printf("Service UUIDs filter: %s\n", strings.Join(filters.ServiceUUIDs, ", "))
			}
		}
	},
}

var bleScanStopCmd = &cobra.Command{
	Use:   "stop-scan [scan_handle]",
	Short: "Stop BLE device scan",
	Long:  "Stop an active BLE device scan.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		scanHandle := args[0]

		err := bleService.StopScan(scanHandle)
		if err != nil {
			fmt.Printf("Error stopping BLE scan: %v\n", err)
			return
		}

		fmt.Printf("BLE scan stopped successfully\n")
		fmt.Printf("Scan handle: %s\n", scanHandle)
	},
}

var bleListCmd = &cobra.Command{
	Use:   "list",
	Short: "List discovered BLE devices",
	Long:  "List all discovered BLE devices from previous scans.",
	Run: func(cmd *cobra.Command, args []string) {
		jsonOutput, _ := cmd.Flags().GetBool("json")

		devices, err := bleService.ListDevices()
		if err != nil {
			fmt.Printf("Error listing BLE devices: %v\n", err)
			return
		}

		if jsonOutput {
			jsonData, _ := json.MarshalIndent(devices, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			if len(devices) == 0 {
				fmt.Println("No BLE devices discovered")
				return
			}

			fmt.Printf("Discovered BLE devices (%d):\n", len(devices))
			for i, device := range devices {
				fmt.Printf("  %d. %s\n", i+1, device.Address)
				if device.Name != "" {
					fmt.Printf("     Name: %s\n", device.Name)
				}
				fmt.Printf("     RSSI: %d dBm\n", device.RSSI)
				if len(device.ServiceUUIDs) > 0 {
					fmt.Printf("     Services: %s\n", strings.Join(device.ServiceUUIDs, ", "))
				}
				fmt.Printf("     Last seen: %s\n", device.LastSeen)
				fmt.Println()
			}
		}
	},
}

var bleConnectCmd = &cobra.Command{
	Use:   "connect [address]",
	Short: "Connect to BLE device",
	Long:  "Connect to a BLE device by its MAC address.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		address := args[0]
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:ble.connect"
		}

		connHandle, err := bleService.Connect(sessionID, caps, address)
		if err != nil {
			fmt.Printf("Error connecting to BLE device: %v\n", err)
			return
		}

		fmt.Printf("Connected to BLE device successfully\n")
		fmt.Printf("Address: %s\n", address)
		fmt.Printf("Connection handle: %s\n", connHandle)
	},
}

var bleDisconnectCmd = &cobra.Command{
	Use:   "disconnect [conn_handle]",
	Short: "Disconnect from BLE device",
	Long:  "Disconnect from a BLE device using its connection handle.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		connHandle := args[0]

		err := bleService.Disconnect(connHandle)
		if err != nil {
			fmt.Printf("Error disconnecting from BLE device: %v\n", err)
			return
		}

		fmt.Printf("Disconnected from BLE device successfully\n")
		fmt.Printf("Connection handle: %s\n", connHandle)
	},
}

var bleReadCmd = &cobra.Command{
	Use:   "read [conn_handle] [service_uuid:char_uuid]",
	Short: "Read GATT characteristic",
	Long:  "Read a GATT characteristic value from a connected BLE device.",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		connHandle := args[0]
		uuidPair := args[1]

		parts := strings.Split(uuidPair, ":")
		if len(parts) != 2 {
			fmt.Printf("Error: UUID format should be 'service_uuid:char_uuid'\n")
			return
		}

		serviceUUID := parts[0]
		charUUID := parts[1]

		data, err := bleService.GattRead(connHandle, serviceUUID, charUUID)
		if err != nil {
			fmt.Printf("Error reading GATT characteristic: %v\n", err)
			return
		}

		fmt.Printf("GATT characteristic read successfully\n")
		fmt.Printf("Service UUID: %s\n", serviceUUID)
		fmt.Printf("Characteristic UUID: %s\n", charUUID)
		fmt.Printf("Data: %x\n", data)
		fmt.Printf("Data (hex): %s\n", fmt.Sprintf("%x", data))
	},
}

var bleWriteCmd = &cobra.Command{
	Use:   "write [conn_handle] [service_uuid:char_uuid] [data]",
	Short: "Write GATT characteristic",
	Long:  "Write data to a GATT characteristic on a connected BLE device.",
	Args:  cobra.ExactArgs(3),
	Run: func(cmd *cobra.Command, args []string) {
		connHandle := args[0]
		uuidPair := args[1]
		dataHex := args[2]

		parts := strings.Split(uuidPair, ":")
		if len(parts) != 2 {
			fmt.Printf("Error: UUID format should be 'service_uuid:char_uuid'\n")
			return
		}

		serviceUUID := parts[0]
		charUUID := parts[1]

		// Parse hex data
		data, err := hexStringToBytes(dataHex)
		if err != nil {
			fmt.Printf("Error parsing hex data: %v\n", err)
			return
		}

		err = bleService.GattWrite(connHandle, serviceUUID, charUUID, data)
		if err != nil {
			fmt.Printf("Error writing GATT characteristic: %v\n", err)
			return
		}

		fmt.Printf("GATT characteristic written successfully\n")
		fmt.Printf("Service UUID: %s\n", serviceUUID)
		fmt.Printf("Characteristic UUID: %s\n", charUUID)
		fmt.Printf("Data: %x\n", data)
	},
}

var bleSubscribeCmd = &cobra.Command{
	Use:   "subscribe [conn_handle] [service_uuid:char_uuid]",
	Short: "Subscribe to GATT notifications",
	Long:  "Subscribe to notifications from a GATT characteristic on a connected BLE device.",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		connHandle := args[0]
		uuidPair := args[1]
		outputFile, _ := cmd.Flags().GetString("out")

		parts := strings.Split(uuidPair, ":")
		if len(parts) != 2 {
			fmt.Printf("Error: UUID format should be 'service_uuid:char_uuid'\n")
			return
		}

		serviceUUID := parts[0]
		charUUID := parts[1]

		notifyHandle, err := bleService.Subscribe(connHandle, serviceUUID, charUUID)
		if err != nil {
			fmt.Printf("Error subscribing to GATT notifications: %v\n", err)
			return
		}

		fmt.Printf("Subscribed to GATT notifications successfully\n")
		fmt.Printf("Service UUID: %s\n", serviceUUID)
		fmt.Printf("Characteristic UUID: %s\n", charUUID)
		fmt.Printf("Notification handle: %s\n", notifyHandle)

		if outputFile != "" {
			fmt.Printf("Notifications will be written to: %s\n", outputFile)
		}
	},
}

var bleUnsubscribeCmd = &cobra.Command{
	Use:   "unsubscribe [notify_handle]",
	Short: "Unsubscribe from GATT notifications",
	Long:  "Unsubscribe from notifications using the notification handle.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		notifyHandle := args[0]

		err := bleService.Unsubscribe(notifyHandle)
		if err != nil {
			fmt.Printf("Error unsubscribing from GATT notifications: %v\n", err)
			return
		}

		fmt.Printf("Unsubscribed from GATT notifications successfully\n")
		fmt.Printf("Notification handle: %s\n", notifyHandle)
	},
}

// Mock BLE service implementation
func (bs *MockBleService) StartScan(sessionID, caps string, filters BleScanFilters) (string, error) {
	scanHandle := fmt.Sprintf("scan_%d", time.Now().UnixNano())
	
	scan := BleScan{
		ScanHandle:   scanHandle,
		SessionID:    sessionID,
		Filters:      filters,
		StartTime:    time.Now(),
		DevicesFound: make([]BleDeviceInfo, 0),
	}
	
	bs.activeScans[scanHandle] = scan
	
	// Generate mock devices
	mockDevices := []BleDeviceInfo{
		{
			Address:      "AA:BB:CC:DD:EE:01",
			Name:         "AethDevice_1",
			RSSI:         -45,
			ServiceUUIDs: []string{"180D", "180F"},
			LastSeen:     time.Now().Format(time.RFC3339),
		},
		{
			Address:      "AA:BB:CC:DD:EE:02",
			Name:         "AethDevice_2",
			RSSI:         -52,
			ServiceUUIDs: []string{"180D", "180F"},
			LastSeen:     time.Now().Format(time.RFC3339),
		},
		{
			Address:      "AA:BB:CC:DD:EE:03",
			Name:         "TestDevice",
			RSSI:         -60,
			ServiceUUIDs: []string{"180A", "180F"},
			LastSeen:     time.Now().Format(time.RFC3339),
		},
	}
	
	// Apply filters
	for _, device := range mockDevices {
		matches := true
		
		if filters.NamePrefix != "" && !strings.HasPrefix(device.Name, filters.NamePrefix) {
			matches = false
		}
		
		if filters.RSSIThreshold != 0 && device.RSSI < filters.RSSIThreshold {
			matches = false
		}
		
		if len(filters.ServiceUUIDs) > 0 {
			hasService := false
			for _, filterUUID := range filters.ServiceUUIDs {
				for _, deviceUUID := range device.ServiceUUIDs {
					if filterUUID == deviceUUID {
						hasService = true
						break
					}
				}
				if hasService {
					break
				}
			}
			if !hasService {
				matches = false
			}
		}
		
		if matches {
			bs.discoveredDevices = append(bs.discoveredDevices, device)
			scan.DevicesFound = append(scan.DevicesFound, device)
		}
	}
	
	bs.activeScans[scanHandle] = scan
	
	return scanHandle, nil
}

func (bs *MockBleService) StopScan(scanHandle string) error {
	delete(bs.activeScans, scanHandle)
	return nil
}

func (bs *MockBleService) ListDevices() ([]BleDeviceInfo, error) {
	return bs.discoveredDevices, nil
}

func (bs *MockBleService) Connect(sessionID, caps, address string) (string, error) {
	connHandle := fmt.Sprintf("conn_%d", time.Now().UnixNano())
	
	connection := BleConnection{
		ConnHandle:  connHandle,
		Address:     address,
		ConnectedAt: time.Now(),
		Services: []BleService{
			{
				UUID: "180D", // Heart Rate
				Characteristics: []BleCharacteristic{
					{
						UUID: "2A37", // Heart Rate Measurement
						Properties: BleCharProperties{
							Read:   true,
							Write:  false,
							Notify: true,
							Indicate: false,
						},
						Value: nil,
					},
				},
			},
			{
				UUID: "180F", // Battery
				Characteristics: []BleCharacteristic{
					{
						UUID: "2A19", // Battery Level
						Properties: BleCharProperties{
							Read:   true,
							Write:  false,
							Notify: false,
							Indicate: false,
						},
						Value: []byte{85}, // 85% battery
					},
				},
			},
		},
	}
	
	bs.activeConnections[connHandle] = connection
	
	return connHandle, nil
}

func (bs *MockBleService) Disconnect(connHandle string) error {
	delete(bs.activeConnections, connHandle)
	return nil
}

func (bs *MockBleService) GattRead(connHandle, serviceUUID, charUUID string) ([]byte, error) {
	connection, exists := bs.activeConnections[connHandle]
	if !exists {
		return nil, fmt.Errorf("connection not found: %s", connHandle)
	}
	
	for _, service := range connection.Services {
		if service.UUID == serviceUUID {
			for _, char := range service.Characteristics {
				if char.UUID == charUUID {
					return char.Value, nil
				}
			}
		}
	}
	
	return nil, fmt.Errorf("characteristic not found: %s:%s", serviceUUID, charUUID)
}

func (bs *MockBleService) GattWrite(connHandle, serviceUUID, charUUID string, data []byte) error {
	connection, exists := bs.activeConnections[connHandle]
	if !exists {
		return fmt.Errorf("connection not found: %s", connHandle)
	}
	
	for _, service := range connection.Services {
		if service.UUID == serviceUUID {
			for _, char := range service.Characteristics {
				if char.UUID == charUUID {
					if !char.Properties.Write {
						return fmt.Errorf("characteristic is not writable")
					}
					// In mock implementation, we just return success
					return nil
				}
			}
		}
	}
	
	return fmt.Errorf("characteristic not found: %s:%s", serviceUUID, charUUID)
}

func (bs *MockBleService) Subscribe(connHandle, serviceUUID, charUUID string) (string, error) {
	connection, exists := bs.activeConnections[connHandle]
	if !exists {
		return "", fmt.Errorf("connection not found: %s", connHandle)
	}
	
	for _, service := range connection.Services {
		if service.UUID == serviceUUID {
			for _, char := range service.Characteristics {
				if char.UUID == charUUID {
					if !char.Properties.Notify {
						return "", fmt.Errorf("characteristic does not support notifications")
					}
					
					notifyHandle := fmt.Sprintf("notify_%d", time.Now().UnixNano())
					
					stream := BleNotificationStream{
						NotifyHandle:      notifyHandle,
						ConnHandle:        connHandle,
						ServiceUUID:       serviceUUID,
						CharUUID:          charUUID,
						StartTime:         time.Now(),
						NotificationCount: 0,
					}
					
					bs.activeNotifications[notifyHandle] = stream
					
					return notifyHandle, nil
				}
			}
		}
	}
	
	return "", fmt.Errorf("characteristic not found: %s:%s", serviceUUID, charUUID)
}

func (bs *MockBleService) Unsubscribe(notifyHandle string) error {
	delete(bs.activeNotifications, notifyHandle)
	return nil
}

// Helper function to convert hex string to bytes
func hexStringToBytes(hexStr string) ([]byte, error) {
	// Remove any spaces or common prefixes
	hexStr = strings.ReplaceAll(hexStr, " ", "")
	hexStr = strings.ReplaceAll(hexStr, "0x", "")
	hexStr = strings.ReplaceAll(hexStr, "0X", "")
	
	// Ensure even length
	if len(hexStr)%2 != 0 {
		hexStr = "0" + hexStr
	}
	
	result := make([]byte, len(hexStr)/2)
	for i := 0; i < len(hexStr); i += 2 {
		val, err := strconv.ParseUint(hexStr[i:i+2], 16, 8)
		if err != nil {
			return nil, fmt.Errorf("invalid hex character: %s", hexStr[i:i+2])
		}
		result[i/2] = byte(val)
	}
	
	return result, nil
}

func init() {
	// BLE scan command flags
	bleScanCmd.Flags().String("name-prefix", "", "Filter devices by name prefix")
	bleScanCmd.Flags().String("service-uuids", "", "Filter devices by service UUIDs (comma-separated)")
	bleScanCmd.Flags().Int16("rssi-threshold", -100, "RSSI threshold filter")
	bleScanCmd.Flags().Uint32("timeout", 30000, "Scan timeout in milliseconds")
	bleScanCmd.Flags().Bool("json", false, "Output in JSON format")
	
	// BLE list command flags
	bleListCmd.Flags().Bool("json", false, "Output in JSON format")
	
	// BLE subscribe command flags
	bleSubscribeCmd.Flags().String("out", "", "Output file for notifications (JSONL format)")
	
	// Add subcommands
	bleCmd.AddCommand(bleScanCmd, bleScanStopCmd, bleListCmd, bleConnectCmd, bleDisconnectCmd, bleReadCmd, bleWriteCmd, bleSubscribeCmd, bleUnsubscribeCmd)
}
