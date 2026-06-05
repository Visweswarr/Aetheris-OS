// package main provides command-line interface for Aetheris device management
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"
	"text/tabwriter"

	"github.com/spf13/cobra"
)

// DeviceType represents the type of device
type DeviceType string

const (
	DeviceTypeCamera     DeviceType = "camera"
	DeviceTypeMicrophone DeviceType = "microphone"
	DeviceTypeGPIO       DeviceType = "gpio"
	DeviceTypeADC        DeviceType = "adc"
	DeviceTypeActuator   DeviceType = "actuator"
)

// ProviderType represents the type of provider
type ProviderType string

const (
	ProviderTypeDeterministic ProviderType = "deterministic"
	ProviderTypeLinux         ProviderType = "linux"
	ProviderTypeCustom        ProviderType = "custom"
)

// DeviceInfo represents device information
type DeviceInfo struct {
	DeviceID     string                 `json:"device_id"`
	DeviceType   DeviceType             `json:"device_type"`
	DevicePath   string                 `json:"device_path"`
	Capabilities []string               `json:"capabilities"`
	Provider     ProviderType           `json:"provider"`
	IsAvailable  bool                   `json:"is_available"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// ProviderStatus represents provider status information
type ProviderStatus struct {
	ProviderID   ProviderType `json:"provider_id"`
	IsAvailable  bool         `json:"is_available"`
	DeviceCount  int          `json:"device_count"`
	LastError    string       `json:"last_error,omitempty"`
	Capabilities []string     `json:"capabilities"`
}

// devicesCmd represents the devices command
var devicesCmd = &cobra.Command{
	Use:   "devices",
	Short: "Manage and list available devices",
	Long:  `Discover, list, and manage available devices across different providers.`,
}

// listCmd represents the list command
var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List available devices",
	Long:  `List all available devices or filter by type/provider.`,
	RunE:  runListDevices,
}

// providerCmd represents the provider command
var providerCmd = &cobra.Command{
	Use:   "provider",
	Short: "Manage device providers",
	Long:  `Get and set the default device provider.`,
}

// providerStatusCmd represents the provider status command
var providerStatusCmd = &cobra.Command{
	Use:   "status",
	Short: "Show provider status",
	Long:  `Show the status of all available providers.`,
	RunE:  runProviderStatus,
}

// providerSetCmd represents the provider set command
var providerSetCmd = &cobra.Command{
	Use:   "set [provider]",
	Short: "Set default provider",
	Long:  `Set the default device provider (deterministic, linux, custom).`,
	Args:  cobra.ExactArgs(1),
	RunE:  runSetProvider,
}

// providerGetCmd represents the provider get command
var providerGetCmd = &cobra.Command{
	Use:   "get",
	Short: "Get current default provider",
	Long:  `Get the current default device provider.`,
	RunE:  runGetProvider,
}

// Command line flags
var (
	deviceTypeFilter   string
	providerTypeFilter string
	outputFormat       string
	showUnavailable    bool
)

func init() {
	rootCmd.AddCommand(devicesCmd)
	devicesCmd.AddCommand(listCmd)
	devicesCmd.AddCommand(providerCmd)
	providerCmd.AddCommand(providerStatusCmd)
	providerCmd.AddCommand(providerSetCmd)
	providerCmd.AddCommand(providerGetCmd)

	// List command flags
	listCmd.Flags().StringVarP(&deviceTypeFilter, "type", "t", "", "Filter by device type (camera, microphone, gpio, adc, actuator)")
	listCmd.Flags().StringVarP(&providerTypeFilter, "provider", "p", "", "Filter by provider type (deterministic, linux, custom)")
	listCmd.Flags().StringVarP(&outputFormat, "format", "f", "table", "Output format (table, json)")
	listCmd.Flags().BoolVar(&showUnavailable, "show-unavailable", false, "Show unavailable devices")
}

// runListDevices executes the list devices command
func runListDevices(cmd *cobra.Command, args []string) error {
	// In a real implementation, this would call the Rust FFI
	// For now, we'll simulate device discovery
	devices, err := discoverDevices()
	if err != nil {
		return fmt.Errorf("failed to discover devices: %w", err)
	}

	// Apply filters
	devices = filterDevices(devices)

	// Output results
	switch outputFormat {
	case "json":
		return outputDevicesJSON(devices)
	case "table":
		return outputDevicesTable(devices)
	default:
		return fmt.Errorf("unsupported output format: %s", outputFormat)
	}
}

// runProviderStatus executes the provider status command
func runProviderStatus(cmd *cobra.Command, args []string) error {
	// In a real implementation, this would call the Rust FFI
	// For now, we'll simulate provider status
	statuses, err := getProviderStatus()
	if err != nil {
		return fmt.Errorf("failed to get provider status: %w", err)
	}

	// Output results
	switch outputFormat {
	case "json":
		return outputProviderStatusJSON(statuses)
	case "table":
		return outputProviderStatusTable(statuses)
	default:
		return fmt.Errorf("unsupported output format: %s", outputFormat)
	}
}

// runSetProvider executes the set provider command
func runSetProvider(cmd *cobra.Command, args []string) error {
	providerStr := args[0]
	provider, err := parseProviderType(providerStr)
	if err != nil {
		return fmt.Errorf("invalid provider type: %s", providerStr)
	}

	// In a real implementation, this would call the Rust FFI
	err = setDefaultProvider(provider)
	if err != nil {
		return fmt.Errorf("failed to set default provider: %w", err)
	}

	fmt.Printf("Default provider set to: %s\n", provider)
	return nil
}

// runGetProvider executes the get provider command
func runGetProvider(cmd *cobra.Command, args []string) error {
	// In a real implementation, this would call the Rust FFI
	// For now, we'll simulate getting the current provider
	provider, err := getDefaultProvider()
	if err != nil {
		return fmt.Errorf("failed to get default provider: %w", err)
	}

	fmt.Printf("Current default provider: %s\n", provider)
	return nil
}

// discoverDevices simulates device discovery
func discoverDevices() ([]DeviceInfo, error) {
	// In a real implementation, this would call the Rust FFI
	// For now, return mock devices
	devices := []DeviceInfo{
		{
			DeviceID:     "camera_mock_0",
			DeviceType:   DeviceTypeCamera,
			DevicePath:   "/dev/video_mock_0",
			Capabilities: []string{"capture", "yuv420", "rgb24"},
			Provider:     ProviderTypeDeterministic,
			IsAvailable:  true,
			Metadata: map[string]interface{}{
				"width": 640,
				"height": 480,
				"fps": 30,
			},
		},
		{
			DeviceID:     "mic_mock_0",
			DeviceType:   DeviceTypeMicrophone,
			DevicePath:   "/dev/audio_mock_0",
			Capabilities: []string{"capture", "s16le", "f32le"},
			Provider:     ProviderTypeDeterministic,
			IsAvailable:  true,
			Metadata: map[string]interface{}{
				"sample_rate": 44100,
				"channels": 2,
			},
		},
		{
			DeviceID:     "gpio_mock_0",
			DeviceType:   DeviceTypeGPIO,
			DevicePath:   "/dev/gpiochip_mock_0",
			Capabilities: []string{"read", "write", "interrupt"},
			Provider:     ProviderTypeDeterministic,
			IsAvailable:  true,
			Metadata: map[string]interface{}{
				"lines": 32,
			},
		},
		{
			DeviceID:     "adc_mock_0",
			DeviceType:   DeviceTypeADC,
			DevicePath:   "/dev/iio_mock_0",
			Capabilities: []string{"sample", "continuous"},
			Provider:     ProviderTypeDeterministic,
			IsAvailable:  true,
			Metadata: map[string]interface{}{
				"channels": 8,
				"resolution": 12,
				"reference_voltage": 3.3,
			},
		},
		{
			DeviceID:     "actuator_mock_0",
			DeviceType:   DeviceTypeActuator,
			DevicePath:   "/dev/pwm_mock_0",
			Capabilities: []string{"pwm", "led", "motor"},
			Provider:     ProviderTypeDeterministic,
			IsAvailable:  true,
			Metadata: map[string]interface{}{
				"channels": 4,
				"frequency_range": "1-1000000",
			},
		},
	}

	// Add Linux devices if available
	if isLinuxProviderAvailable() {
		linuxDevices := []DeviceInfo{
			{
				DeviceID:     "/dev/video0",
				DeviceType:   DeviceTypeCamera,
				DevicePath:   "/dev/video0",
				Capabilities: []string{"capture", "yuv420", "rgb24", "mjpeg"},
				Provider:     ProviderTypeLinux,
				IsAvailable:  true,
				Metadata: map[string]interface{}{
					"driver": "uvcvideo",
					"card": "USB2.0 Camera",
				},
			},
			{
				DeviceID:     "hw:0,0",
				DeviceType:   DeviceTypeMicrophone,
				DevicePath:   "hw:0,0",
				Capabilities: []string{"capture", "s16le", "f32le"},
				Provider:     ProviderTypeLinux,
				IsAvailable:  true,
				Metadata: map[string]interface{}{
					"card_name": "HDA Intel PCH",
					"device_type": "alsa_card",
				},
			},
			{
				DeviceID:     "/dev/gpiochip0",
				DeviceType:   DeviceTypeGPIO,
				DevicePath:   "/dev/gpiochip0",
				Capabilities: []string{"read", "write", "interrupt", "pull_up", "pull_down"},
				Provider:     ProviderTypeLinux,
				IsAvailable:  true,
				Metadata: map[string]interface{}{
					"chip_name": "gpiochip0",
					"line_count": 32,
				},
			},
		}
		devices = append(devices, linuxDevices...)
	}

	return devices, nil
}

// filterDevices applies command line filters to the device list
func filterDevices(devices []DeviceInfo) []DeviceInfo {
	var filtered []DeviceInfo

	for _, device := range devices {
		// Filter by device type
		if deviceTypeFilter != "" && string(device.DeviceType) != deviceTypeFilter {
			continue
		}

		// Filter by provider type
		if providerTypeFilter != "" && string(device.Provider) != providerTypeFilter {
			continue
		}

		// Filter by availability
		if !showUnavailable && !device.IsAvailable {
			continue
		}

		filtered = append(filtered, device)
	}

	return filtered
}

// outputDevicesTable outputs devices in table format
func outputDevicesTable(devices []DeviceInfo) error {
	if len(devices) == 0 {
		fmt.Println("No devices found")
		return nil
	}

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(w, "DEVICE ID\tTYPE\tPROVIDER\tPATH\tAVAILABLE\tCAPABILITIES")
	fmt.Fprintln(w, "---------\t----\t--------\t----\t---------\t------------")

	for _, device := range devices {
		available := "Yes"
		if !device.IsAvailable {
			available = "No"
		}
		capabilities := strings.Join(device.Capabilities, ", ")
		fmt.Fprintf(w, "%s\t%s\t%s\t%s\t%s\t%s\n",
			device.DeviceID,
			device.DeviceType,
			device.Provider,
			device.DevicePath,
			available,
			capabilities,
		)
	}

	return w.Flush()
}

// outputDevicesJSON outputs devices in JSON format
func outputDevicesJSON(devices []DeviceInfo) error {
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetIndent("", "  ")
	return encoder.Encode(devices)
}

// getProviderStatus simulates getting provider status
func getProviderStatus() ([]ProviderStatus, error) {
	// In a real implementation, this would call the Rust FFI
	statuses := []ProviderStatus{
		{
			ProviderID:   ProviderTypeDeterministic,
			IsAvailable:  true,
			DeviceCount:  5,
			Capabilities: []string{"camera.capture", "microphone.capture", "gpio.read", "gpio.write", "adc.sample", "actuator.control", "deterministic"},
		},
	}

	if isLinuxProviderAvailable() {
		statuses = append(statuses, ProviderStatus{
			ProviderID:   ProviderTypeLinux,
			IsAvailable:  true,
			DeviceCount:  3,
			Capabilities: []string{"camera.capture", "microphone.capture", "gpio.read", "gpio.write", "gpio.interrupt", "adc.sample", "adc.continuous", "actuator.pwm", "actuator.led", "actuator.motor", "hardware"},
		})
	}

	return statuses, nil
}

// outputProviderStatusTable outputs provider status in table format
func outputProviderStatusTable(statuses []ProviderStatus) error {
	if len(statuses) == 0 {
		fmt.Println("No providers found")
		return nil
	}

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(w, "PROVIDER\tAVAILABLE\tDEVICE COUNT\tCAPABILITIES")
	fmt.Fprintln(w, "--------\t---------\t------------\t------------")

	for _, status := range statuses {
		available := "Yes"
		if !status.IsAvailable {
			available = "No"
		}
		capabilities := strings.Join(status.Capabilities, ", ")
		fmt.Fprintf(w, "%s\t%s\t%d\t%s\n",
			status.ProviderID,
			available,
			status.DeviceCount,
			capabilities,
		)
	}

	return w.Flush()
}

// outputProviderStatusJSON outputs provider status in JSON format
func outputProviderStatusJSON(statuses []ProviderStatus) error {
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetIndent("", "  ")
	return encoder.Encode(statuses)
}

// setDefaultProvider simulates setting the default provider
func setDefaultProvider(provider ProviderType) error {
	// In a real implementation, this would call the Rust FFI
	fmt.Printf("Setting default provider to: %s\n", provider)
	return nil
}

// getDefaultProvider simulates getting the default provider
func getDefaultProvider() (ProviderType, error) {
	// In a real implementation, this would call the Rust FFI
	return ProviderTypeDeterministic, nil
}

// parseProviderType parses a provider type string
func parseProviderType(providerStr string) (ProviderType, error) {
	switch strings.ToLower(providerStr) {
	case "deterministic", "det":
		return ProviderTypeDeterministic, nil
	case "linux":
		return ProviderTypeLinux, nil
	case "custom":
		return ProviderTypeCustom, nil
	default:
		return "", fmt.Errorf("unknown provider type: %s", providerStr)
	}
}

// isLinuxProviderAvailable checks if Linux provider is available
func isLinuxProviderAvailable() bool {
	// In a real implementation, this would check system capabilities
	// For now, always return true for demonstration
	return true
}

// Helper functions for device type validation
func isValidDeviceType(deviceType string) bool {
	switch deviceType {
	case "camera", "microphone", "gpio", "adc", "actuator":
		return true
	default:
		return false
	}
}

func isValidProviderType(providerType string) bool {
	switch providerType {
	case "deterministic", "det", "linux", "custom":
		return true
	default:
		return false
	}
}
