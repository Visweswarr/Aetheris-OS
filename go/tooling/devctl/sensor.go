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

// Sensor kind enumeration
type SensorKind string

const (
	SensorAccelerometer SensorKind = "accelerometer"
	SensorGyroscope     SensorKind = "gyroscope"
	SensorMagnetometer  SensorKind = "magnetometer"
	SensorTemperature   SensorKind = "temperature"
	SensorHumidity      SensorKind = "humidity"
	SensorPressure      SensorKind = "pressure"
	SensorLight         SensorKind = "light"
	SensorProximity     SensorKind = "proximity"
	SensorHeartRate     SensorKind = "heart_rate"
	SensorCustom        SensorKind = "custom"
)

// Sensor value type
type SensorValueType string

const (
	SensorValueScalar  SensorValueType = "scalar"
	SensorValueVector2 SensorValueType = "vector2"
	SensorValueVector3 SensorValueType = "vector3"
)

// Sensor value
type SensorValue struct {
	Type  SensorValueType `json:"type"`
	Value interface{}     `json:"value"`
}

// Sensor description
type SensorDesc struct {
	Name         string    `json:"name"`
	Kind         SensorKind `json:"kind"`
	Location     string    `json:"location"`
	Unit         string    `json:"unit"`
	RangeMin     float64   `json:"range_min"`
	RangeMax     float64   `json:"range_max"`
	Resolution   float64   `json:"resolution"`
	SampleRates  []uint32  `json:"sample_rates"`
}

// Sensor information
type SensorInfo struct {
	SensorID       string    `json:"sensor_id"`
	Name           string    `json:"name"`
	Kind           SensorKind `json:"kind"`
	Location       string    `json:"location"`
	Unit           string    `json:"unit"`
	RangeMin       float64   `json:"range_min"`
	RangeMax       float64   `json:"range_max"`
	Resolution     float64   `json:"resolution"`
	SampleRates    []uint32  `json:"sample_rates"`
	RegisteredAt   string    `json:"registered_at"`
	ActiveSampling bool      `json:"active_sampling"`
}

// Sensor sample
type SensorSample struct {
	SensorID       string      `json:"sensor_id"`
	Timestamp      time.Time   `json:"timestamp"`
	Value          SensorValue `json:"value"`
	SequenceNumber uint64      `json:"sequence_number"`
}

// Mock sensor service for demonstration
type MockSensorService struct {
	registeredSensors map[string]SensorInfo
	activeSampling    map[string]SamplingSession
	activePreviews    map[string]PreviewStream
}

type SamplingSession struct {
	Handle        string
	SensorID      string
	SessionID     string
	Hz            uint32
	Seed          uint64
	StartTime     time.Time
	SampleCount   uint64
	Deterministic bool
}

type PreviewStream struct {
	StreamHandle string
	SensorID     string
	HzMax        uint32
	StartTime    time.Time
	SampleCount  uint64
}

var sensorService = &MockSensorService{
	registeredSensors: make(map[string]SensorInfo),
	activeSampling:    make(map[string]SamplingSession),
	activePreviews:    make(map[string]PreviewStream),
}

// Sensor commands
var sensorCmd = &cobra.Command{
	Use:   "sensor",
	Short: "Manage sensor operations",
	Long:  "Manage sensor operations, including registration, sampling, preview, and snapshots.",
}

var sensorListCmd = &cobra.Command{
	Use:   "list",
	Short: "List registered sensors",
	Long:  "List all registered sensors with their capabilities and status.",
	Run: func(cmd *cobra.Command, args []string) {
		jsonOutput, _ := cmd.Flags().GetBool("json")

		sensors, err := sensorService.ListSensors()
		if err != nil {
			fmt.Printf("Error listing sensors: %v\n", err)
			return
		}

		if jsonOutput {
			jsonData, _ := json.MarshalIndent(sensors, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			if len(sensors) == 0 {
				fmt.Println("No sensors registered")
				return
			}

			fmt.Printf("Registered sensors (%d):\n", len(sensors))
			for i, sensor := range sensors {
				fmt.Printf("  %d. %s (%s)\n", i+1, sensor.Name, sensor.SensorID)
				fmt.Printf("     Kind: %s\n", sensor.Kind)
				fmt.Printf("     Location: %s\n", sensor.Location)
				fmt.Printf("     Unit: %s\n", sensor.Unit)
				fmt.Printf("     Range: %.2f - %.2f\n", sensor.RangeMin, sensor.RangeMax)
				fmt.Printf("     Resolution: %.3f\n", sensor.Resolution)
				fmt.Printf("     Sample rates: %s\n", formatSampleRates(sensor.SampleRates))
				fmt.Printf("     Active sampling: %v\n", sensor.ActiveSampling)
				fmt.Printf("     Registered: %s\n", sensor.RegisteredAt)
				fmt.Println()
			}
		}
	},
}

var sensorRegisterCmd = &cobra.Command{
	Use:   "register",
	Short: "Register a new sensor",
	Long:  "Register a new sensor with its capabilities and configuration.",
	Run: func(cmd *cobra.Command, args []string) {
		name, _ := cmd.Flags().GetString("name")
		kindStr, _ := cmd.Flags().GetString("kind")
		location, _ := cmd.Flags().GetString("location")
		unit, _ := cmd.Flags().GetString("unit")
		rangeMin, _ := cmd.Flags().GetFloat64("range-min")
		rangeMax, _ := cmd.Flags().GetFloat64("range-max")
		resolution, _ := cmd.Flags().GetFloat64("resolution")
		sampleRatesStr, _ := cmd.Flags().GetString("sample-rates")

		if name == "" {
			fmt.Printf("Error: sensor name is required\n")
			return
		}

		kind := SensorKind(kindStr)
		if kind == "" {
			kind = SensorCustom
		}

		// Parse sample rates
		var sampleRates []uint32
		if sampleRatesStr != "" {
			rateStrs := strings.Split(sampleRatesStr, ",")
			for _, rateStr := range rateStrs {
				if rate, err := strconv.ParseUint(strings.TrimSpace(rateStr), 10, 32); err == nil {
					sampleRates = append(sampleRates, uint32(rate))
				}
			}
		}

		// Set defaults based on sensor kind
		if location == "" {
			location = getDefaultLocation(kind)
		}
		if unit == "" {
			unit = getDefaultUnit(kind)
		}
		if rangeMin == 0 && rangeMax == 0 {
			rangeMin, rangeMax = getDefaultRange(kind)
		}
		if resolution == 0 {
			resolution = getDefaultResolution(kind)
		}
		if len(sampleRates) == 0 {
			sampleRates = getDefaultSampleRates(kind)
		}

		desc := SensorDesc{
			Name:        name,
			Kind:        kind,
			Location:    location,
			Unit:        unit,
			RangeMin:    rangeMin,
			RangeMax:    rangeMax,
			Resolution:  resolution,
			SampleRates: sampleRates,
		}

		sensorID, err := sensorService.RegisterSensor(desc)
		if err != nil {
			fmt.Printf("Error registering sensor: %v\n", err)
			return
		}

		fmt.Printf("Sensor registered successfully\n")
		fmt.Printf("Sensor ID: %s\n", sensorID)
		fmt.Printf("Name: %s\n", desc.Name)
		fmt.Printf("Kind: %s\n", desc.Kind)
		fmt.Printf("Location: %s\n", desc.Location)
		fmt.Printf("Unit: %s\n", desc.Unit)
		fmt.Printf("Range: %.2f - %.2f\n", desc.RangeMin, desc.RangeMax)
		fmt.Printf("Resolution: %.3f\n", desc.Resolution)
		fmt.Printf("Sample rates: %s\n", formatSampleRates(desc.SampleRates))
	},
}

var sensorStartCmd = &cobra.Command{
	Use:   "start [sensor_id]",
	Short: "Start sensor sampling",
	Long:  "Start sampling from a registered sensor at the specified rate.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sensorID := args[0]
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		hz, _ := cmd.Flags().GetUint32("hz")
		seed, _ := cmd.Flags().GetUint64("seed")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:sensor.sample"
		}

		handle, err := sensorService.StartSampling(sessionID, caps, sensorID, hz, seed)
		if err != nil {
			fmt.Printf("Error starting sensor sampling: %v\n", err)
			return
		}

		fmt.Printf("Sensor sampling started successfully\n")
		fmt.Printf("Sensor ID: %s\n", sensorID)
		fmt.Printf("Handle: %s\n", handle)
		fmt.Printf("Session ID: %s\n", sessionID)
		fmt.Printf("Sample rate: %d Hz\n", hz)
		if seed != 0 {
			fmt.Printf("Deterministic seed: %d\n", seed)
		}
	},
}

var sensorStopCmd = &cobra.Command{
	Use:   "stop [handle]",
	Short: "Stop sensor sampling",
	Long:  "Stop sampling from a sensor using its handle.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		handle := args[0]

		err := sensorService.StopSampling(handle)
		if err != nil {
			fmt.Printf("Error stopping sensor sampling: %v\n", err)
			return
		}

		fmt.Printf("Sensor sampling stopped successfully\n")
		fmt.Printf("Handle: %s\n", handle)
	},
}

var sensorPreviewCmd = &cobra.Command{
	Use:   "preview [sensor_id]",
	Short: "Start sensor preview stream",
	Long:  "Start a preview stream from a sensor at the specified maximum rate.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sensorID := args[0]
		fps, _ := cmd.Flags().GetUint32("fps")

		streamHandle, err := sensorService.StartPreview(sensorID, fps)
		if err != nil {
			fmt.Printf("Error starting sensor preview: %v\n", err)
			return
		}

		fmt.Printf("Sensor preview started successfully\n")
		fmt.Printf("Sensor ID: %s\n", sensorID)
		fmt.Printf("Stream handle: %s\n", streamHandle)
		fmt.Printf("Max rate: %d Hz\n", fps)
	},
}

var sensorSnapshotCmd = &cobra.Command{
	Use:   "snapshot [sensor_id]",
	Short: "Create sensor snapshot",
	Long:  "Create a deterministic snapshot of sensor data and save to NGFS.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sensorID := args[0]
		outputPath, _ := cmd.Flags().GetString("out")

		if outputPath == "" {
			outputPath = fmt.Sprintf("snaps/%s.ngfs", sensorID)
		}

		snapshotID, err := sensorService.CreateSnapshot(sensorID, outputPath)
		if err != nil {
			fmt.Printf("Error creating sensor snapshot: %v\n", err)
			return
		}

		fmt.Printf("Sensor snapshot created successfully\n")
		fmt.Printf("Sensor ID: %s\n", sensorID)
		fmt.Printf("Snapshot ID: %s\n", snapshotID)
		fmt.Printf("Output path: %s\n", outputPath)
	},
}

// Mock sensor service implementation
func (ss *MockSensorService) RegisterSensor(desc SensorDesc) (string, error) {
	sensorID := fmt.Sprintf("sensor_%d", time.Now().UnixNano())
	
	sensorInfo := SensorInfo{
		SensorID:       sensorID,
		Name:           desc.Name,
		Kind:           desc.Kind,
		Location:       desc.Location,
		Unit:           desc.Unit,
		RangeMin:       desc.RangeMin,
		RangeMax:       desc.RangeMax,
		Resolution:     desc.Resolution,
		SampleRates:    desc.SampleRates,
		RegisteredAt:   time.Now().Format(time.RFC3339),
		ActiveSampling: false,
	}
	
	ss.registeredSensors[sensorID] = sensorInfo
	
	return sensorID, nil
}

func (ss *MockSensorService) ListSensors() ([]SensorInfo, error) {
	sensors := make([]SensorInfo, 0, len(ss.registeredSensors))
	for _, sensor := range ss.registeredSensors {
		sensors = append(sensors, sensor)
	}
	return sensors, nil
}

func (ss *MockSensorService) StartSampling(sessionID, caps, sensorID string, hz uint32, seed uint64) (string, error) {
	sensor, exists := ss.registeredSensors[sensorID]
	if !exists {
		return "", fmt.Errorf("sensor not found: %s", sensorID)
	}
	
	// Check if sample rate is supported
	rateSupported := false
	for _, rate := range sensor.SampleRates {
		if rate == hz {
			rateSupported = true
			break
		}
	}
	
	if !rateSupported {
		return "", fmt.Errorf("sample rate %d Hz not supported", hz)
	}
	
	handle := fmt.Sprintf("sampling_%d", time.Now().UnixNano())
	
	session := SamplingSession{
		Handle:        handle,
		SensorID:      sensorID,
		SessionID:     sessionID,
		Hz:            hz,
		Seed:          seed,
		StartTime:     time.Now(),
		SampleCount:   0,
		Deterministic: seed != 0,
	}
	
	ss.activeSampling[handle] = session
	
	// Update sensor status
	sensor.ActiveSampling = true
	ss.registeredSensors[sensorID] = sensor
	
	return handle, nil
}

func (ss *MockSensorService) StopSampling(handle string) error {
	session, exists := ss.activeSampling[handle]
	if !exists {
		return fmt.Errorf("sampling session not found: %s", handle)
	}
	
	// Update sensor status
	if sensor, exists := ss.registeredSensors[session.SensorID]; exists {
		sensor.ActiveSampling = false
		ss.registeredSensors[session.SensorID] = sensor
	}
	
	delete(ss.activeSampling, handle)
	
	return nil
}

func (ss *MockSensorService) StartPreview(sensorID string, hzMax uint32) (string, error) {
	_, exists := ss.registeredSensors[sensorID]
	if !exists {
		return "", fmt.Errorf("sensor not found: %s", sensorID)
	}
	
	streamHandle := fmt.Sprintf("preview_%d", time.Now().UnixNano())
	
	stream := PreviewStream{
		StreamHandle: streamHandle,
		SensorID:     sensorID,
		HzMax:        hzMax,
		StartTime:    time.Now(),
		SampleCount:  0,
	}
	
	ss.activePreviews[streamHandle] = stream
	
	return streamHandle, nil
}

func (ss *MockSensorService) CreateSnapshot(sensorID, outputPath string) (string, error) {
	_, exists := ss.registeredSensors[sensorID]
	if !exists {
		return "", fmt.Errorf("sensor not found: %s", sensorID)
	}
	
	snapshotID := fmt.Sprintf("snapshot_%d", time.Now().UnixNano())
	
	// TODO: Create actual NGFS snapshot
	fmt.Printf("Creating NGFS snapshot at: %s\n", outputPath)
	
	return snapshotID, nil
}

// Helper functions
func formatSampleRates(rates []uint32) string {
	if len(rates) == 0 {
		return "none"
	}
	
	rateStrs := make([]string, len(rates))
	for i, rate := range rates {
		rateStrs[i] = fmt.Sprintf("%d Hz", rate)
	}
	
	return strings.Join(rateStrs, ", ")
}

func getDefaultLocation(kind SensorKind) string {
	switch kind {
	case SensorAccelerometer, SensorGyroscope, SensorMagnetometer:
		return "IMU"
	case SensorTemperature:
		return "CPU"
	case SensorLight:
		return "Ambient"
	case SensorHumidity, SensorPressure:
		return "Environment"
	case SensorProximity:
		return "Front"
	case SensorHeartRate:
		return "Wrist"
	default:
		return "Unknown"
	}
}

func getDefaultUnit(kind SensorKind) string {
	switch kind {
	case SensorAccelerometer:
		return "m/s²"
	case SensorGyroscope:
		return "deg/s"
	case SensorMagnetometer:
		return "μT"
	case SensorTemperature:
		return "°C"
	case SensorHumidity:
		return "%"
	case SensorPressure:
		return "Pa"
	case SensorLight:
		return "lux"
	case SensorProximity:
		return "cm"
	case SensorHeartRate:
		return "bpm"
	default:
		return "units"
	}
}

func getDefaultRange(kind SensorKind) (float64, float64) {
	switch kind {
	case SensorAccelerometer:
		return -20.0, 20.0
	case SensorGyroscope:
		return -1000.0, 1000.0
	case SensorMagnetometer:
		return -100.0, 100.0
	case SensorTemperature:
		return 0.0, 100.0
	case SensorHumidity:
		return 0.0, 100.0
	case SensorPressure:
		return 80000.0, 120000.0
	case SensorLight:
		return 0.0, 1000.0
	case SensorProximity:
		return 0.0, 100.0
	case SensorHeartRate:
		return 30.0, 200.0
	default:
		return 0.0, 100.0
	}
}

func getDefaultResolution(kind SensorKind) float64 {
	switch kind {
	case SensorAccelerometer:
		return 0.01
	case SensorGyroscope:
		return 0.1
	case SensorMagnetometer:
		return 0.1
	case SensorTemperature:
		return 0.1
	case SensorHumidity:
		return 0.1
	case SensorPressure:
		return 1.0
	case SensorLight:
		return 1.0
	case SensorProximity:
		return 0.1
	case SensorHeartRate:
		return 1.0
	default:
		return 1.0
	}
}

func getDefaultSampleRates(kind SensorKind) []uint32 {
	switch kind {
	case SensorAccelerometer:
		return []uint32{10, 50, 100, 200}
	case SensorGyroscope:
		return []uint32{10, 100, 1000}
	case SensorMagnetometer:
		return []uint32{1, 10, 50}
	case SensorTemperature:
		return []uint32{1, 10, 100}
	case SensorHumidity:
		return []uint32{1, 10}
	case SensorPressure:
		return []uint32{1, 10, 50}
	case SensorLight:
		return []uint32{1, 5, 10}
	case SensorProximity:
		return []uint32{1, 10}
	case SensorHeartRate:
		return []uint32{1, 10}
	default:
		return []uint32{1, 10}
	}
}

func init() {
	// Sensor list command flags
	sensorListCmd.Flags().Bool("json", false, "Output in JSON format")
	
	// Sensor register command flags
	sensorRegisterCmd.Flags().String("name", "", "Sensor name (required)")
	sensorRegisterCmd.Flags().String("kind", "", "Sensor kind (accelerometer, gyroscope, temperature, etc.)")
	sensorRegisterCmd.Flags().String("location", "", "Sensor location")
	sensorRegisterCmd.Flags().String("unit", "", "Sensor unit")
	sensorRegisterCmd.Flags().Float64("range-min", 0, "Minimum value range")
	sensorRegisterCmd.Flags().Float64("range-max", 0, "Maximum value range")
	sensorRegisterCmd.Flags().Float64("resolution", 0, "Sensor resolution")
	sensorRegisterCmd.Flags().String("sample-rates", "", "Supported sample rates (comma-separated)")
	
	// Sensor start command flags
	sensorStartCmd.Flags().Uint32("hz", 10, "Sample rate in Hz")
	sensorStartCmd.Flags().Uint64("seed", 0, "Deterministic seed (0 for non-deterministic)")
	
	// Sensor preview command flags
	sensorPreviewCmd.Flags().Uint32("fps", 5, "Maximum preview rate in Hz")
	
	// Sensor snapshot command flags
	sensorSnapshotCmd.Flags().String("out", "", "Output NGFS path")
	
	// Add subcommands
	sensorCmd.AddCommand(sensorListCmd, sensorRegisterCmd, sensorStartCmd, sensorStopCmd, sensorPreviewCmd, sensorSnapshotCmd)
}
