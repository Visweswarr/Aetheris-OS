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

// GPIO pin mode enumeration
type GpioMode string

const (
	GpioModeInput  GpioMode = "input"
	GpioModeOutput GpioMode = "output"
)

// GPIO pull configuration
type GpioPull string

const (
	GpioPullNone   GpioPull = "none"
	GpioPullUp     GpioPull = "up"
	GpioPullDown   GpioPull = "down"
)

// GPIO configuration
type GpioConfig struct {
	Pin           uint32   `json:"pin"`
	Mode          GpioMode `json:"mode"`
	Pull          GpioPull `json:"pull"`
	InitialValue  bool     `json:"initial_value"`
	DebounceMs    uint32   `json:"debounce_ms"`
}

// GPIO pin state
type GpioState struct {
	Pin           uint32   `json:"pin"`
	Mode          GpioMode `json:"mode"`
	Pull          GpioPull `json:"pull"`
	Value         bool     `json:"value"`
	LastChange    uint64   `json:"last_change"`
	ChangeCount   uint64   `json:"change_count"`
}

// GPIO operation result
type GpioOperationResult struct {
	OperationID   string    `json:"operation_id"`
	Pin           uint32    `json:"pin"`
	Operation     string    `json:"operation"`
	Result        bool      `json:"result"`
	Timestamp     uint64    `json:"timestamp"`
	Deterministic bool      `json:"deterministic"`
}

// Mock GPIO service for demonstration
type MockGpioService struct {
	configuredPins map[uint32]GpioState
	operationHistory []GpioOperationResult
	tickCounter    uint64
	deterministicMode bool
}

var gpioService = &MockGpioService{
	configuredPins: make(map[uint32]GpioState),
	operationHistory: make([]GpioOperationResult, 0),
	tickCounter:    0,
	deterministicMode: false,
}

// GPIO commands
var gpioCmd = &cobra.Command{
	Use:   "gpio",
	Short: "Manage GPIO operations",
	Long:  "Manage GPIO operations, including pin configuration, digital I/O, and deterministic state management.",
}

var gpioConfigureCmd = &cobra.Command{
	Use:   "configure",
	Short: "Configure GPIO pin",
	Long:  "Configure a GPIO pin with mode, pull, and initial value settings.",
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		pin, _ := cmd.Flags().GetUint32("pin")
		modeStr, _ := cmd.Flags().GetString("mode")
		pullStr, _ := cmd.Flags().GetString("pull")
		initialValue, _ := cmd.Flags().GetBool("initial-value")
		debounceMs, _ := cmd.Flags().GetUint32("debounce")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:gpio.configure"
		}

		mode := GpioMode(modeStr)
		if mode == "" {
			mode = GpioModeInput
		}

		pull := GpioPull(pullStr)
		if pull == "" {
			pull = GpioPullNone
		}

		config := GpioConfig{
			Pin:          pin,
			Mode:         mode,
			Pull:         pull,
			InitialValue: initialValue,
			DebounceMs:   debounceMs,
		}

		err := gpioService.ConfigurePin(sessionID, caps, config)
		if err != nil {
			fmt.Printf("Error configuring GPIO pin: %v\n", err)
			return
		}

		fmt.Printf("GPIO pin configured successfully\n")
		fmt.Printf("Pin: %d\n", config.Pin)
		fmt.Printf("Mode: %s\n", config.Mode)
		fmt.Printf("Pull: %s\n", config.Pull)
		fmt.Printf("Initial value: %v\n", config.InitialValue)
		fmt.Printf("Debounce: %d ms\n", config.DebounceMs)
	},
}

var gpioReadCmd = &cobra.Command{
	Use:   "read [pin]",
	Short: "Read GPIO pin value",
	Long:  "Read the current digital value from a configured GPIO pin.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		jsonOutput, _ := cmd.Flags().GetBool("json")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:gpio.read"
		}

		pin, err := strconv.ParseUint(args[0], 10, 32)
		if err != nil {
			fmt.Printf("Error: invalid pin number: %v\n", err)
			return
		}

		value, err := gpioService.ReadPin(sessionID, caps, uint32(pin))
		if err != nil {
			fmt.Printf("Error reading GPIO pin: %v\n", err)
			return
		}

		if jsonOutput {
			result := map[string]interface{}{
				"pin":   uint32(pin),
				"value": value,
			}
			jsonData, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			fmt.Printf("GPIO pin %d value: %v\n", pin, value)
		}
	},
}

var gpioWriteCmd = &cobra.Command{
	Use:   "write [pin] [value]",
	Short: "Write GPIO pin value",
	Long:  "Write a digital value to a configured GPIO pin.",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:gpio.write"
		}

		pin, err := strconv.ParseUint(args[0], 10, 32)
		if err != nil {
			fmt.Printf("Error: invalid pin number: %v\n", err)
			return
		}

		value, err := strconv.ParseBool(args[1])
		if err != nil {
			fmt.Printf("Error: invalid value (must be true/false): %v\n", err)
			return
		}

		err = gpioService.WritePin(sessionID, caps, uint32(pin), value)
		if err != nil {
			fmt.Printf("Error writing GPIO pin: %v\n", err)
			return
		}

		fmt.Printf("GPIO pin %d set to %v\n", pin, value)
	},
}

var gpioToggleCmd = &cobra.Command{
	Use:   "toggle [pin]",
	Short: "Toggle GPIO pin value",
	Long:  "Toggle the current value of a configured GPIO pin.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		jsonOutput, _ := cmd.Flags().GetBool("json")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:gpio.write"
		}

		pin, err := strconv.ParseUint(args[0], 10, 32)
		if err != nil {
			fmt.Printf("Error: invalid pin number: %v\n", err)
			return
		}

		newValue, err := gpioService.TogglePin(sessionID, caps, uint32(pin))
		if err != nil {
			fmt.Printf("Error toggling GPIO pin: %v\n", err)
			return
		}

		if jsonOutput {
			result := map[string]interface{}{
				"pin":       uint32(pin),
				"new_value": newValue,
			}
			jsonData, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			fmt.Printf("GPIO pin %d toggled to %v\n", pin, newValue)
		}
	},
}

var gpioListCmd = &cobra.Command{
	Use:   "list",
	Short: "List configured GPIO pins",
	Long:  "List all configured GPIO pins with their current state.",
	Run: func(cmd *cobra.Command, args []string) {
		jsonOutput, _ := cmd.Flags().GetBool("json")

		pins, err := gpioService.ListConfiguredPins()
		if err != nil {
			fmt.Printf("Error listing GPIO pins: %v\n", err)
			return
		}

		if jsonOutput {
			jsonData, _ := json.MarshalIndent(pins, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			if len(pins) == 0 {
				fmt.Println("No GPIO pins configured")
				return
			}

			fmt.Printf("Configured GPIO pins (%d):\n", len(pins))
			for i, pin := range pins {
				fmt.Printf("  %d. Pin %d\n", i+1, pin.Pin)
				fmt.Printf("     Mode: %s\n", pin.Mode)
				fmt.Printf("     Pull: %s\n", pin.Pull)
				fmt.Printf("     Value: %v\n", pin.Value)
				fmt.Printf("     Last change: %d\n", pin.LastChange)
				fmt.Printf("     Change count: %d\n", pin.ChangeCount)
				fmt.Println()
			}
		}
	},
}

var gpioHistoryCmd = &cobra.Command{
	Use:   "history",
	Short: "Show GPIO operation history",
	Long:  "Show the history of GPIO operations with timestamps and deterministic flags.",
	Run: func(cmd *cobra.Command, args []string) {
		limit, _ := cmd.Flags().GetInt("limit")
		jsonOutput, _ := cmd.Flags().GetBool("json")

		operations, err := gpioService.GetOperationHistory(limit)
		if err != nil {
			fmt.Printf("Error getting GPIO operation history: %v\n", err)
			return
		}

		if jsonOutput {
			jsonData, _ := json.MarshalIndent(operations, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			if len(operations) == 0 {
				fmt.Println("No GPIO operations in history")
				return
			}

			fmt.Printf("GPIO operation history (%d operations):\n", len(operations))
			for i, op := range operations {
				fmt.Printf("  %d. %s\n", i+1, op.OperationID)
				fmt.Printf("     Pin: %d\n", op.Pin)
				fmt.Printf("     Operation: %s\n", op.Operation)
				fmt.Printf("     Result: %v\n", op.Result)
				fmt.Printf("     Timestamp: %d\n", op.Timestamp)
				fmt.Printf("     Deterministic: %v\n", op.Deterministic)
				fmt.Println()
			}
		}
	},
}

var gpioSnapshotCmd = &cobra.Command{
	Use:   "snapshot",
	Short: "Create GPIO snapshot",
	Long:  "Create a deterministic snapshot of GPIO state and save to NGFS.",
	Run: func(cmd *cobra.Command, args []string) {
		outputPath, _ := cmd.Flags().GetString("out")

		if outputPath == "" {
			outputPath = fmt.Sprintf("snaps/gpio_%d.ngfs", time.Now().Unix())
		}

		snapshotID, err := gpioService.CreateSnapshot(outputPath)
		if err != nil {
			fmt.Printf("Error creating GPIO snapshot: %v\n", err)
			return
		}

		fmt.Printf("GPIO snapshot created successfully\n")
		fmt.Printf("Snapshot ID: %s\n", snapshotID)
		fmt.Printf("Output path: %s\n", outputPath)
	},
}

var gpioDeterministicCmd = &cobra.Command{
	Use:   "deterministic [enable|disable|status]",
	Short: "Manage deterministic mode",
	Long:  "Enable, disable, or check the status of deterministic mode for GPIO operations.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		action := args[0]

		switch action {
		case "enable":
			err := gpioService.EnableDeterministicMode()
			if err != nil {
				fmt.Printf("Error enabling deterministic mode: %v\n", err)
				return
			}
			fmt.Println("Deterministic mode enabled")
		case "disable":
			err := gpioService.DisableDeterministicMode()
			if err != nil {
				fmt.Printf("Error disabling deterministic mode: %v\n", err)
				return
			}
			fmt.Println("Deterministic mode disabled")
		case "status":
			enabled, tickCount, err := gpioService.GetDeterministicStatus()
			if err != nil {
				fmt.Printf("Error getting deterministic status: %v\n", err)
				return
			}
			fmt.Printf("Deterministic mode: %v\n", enabled)
			fmt.Printf("Tick counter: %d\n", tickCount)
		default:
			fmt.Printf("Error: invalid action '%s'. Use 'enable', 'disable', or 'status'\n", action)
		}
	},
}

// Mock GPIO service implementation
func (gs *MockGpioService) ConfigurePin(sessionID, caps string, config GpioConfig) error {
	// Validate configuration
	if config.Mode != GpioModeInput && config.Mode != GpioModeOutput {
		return fmt.Errorf("invalid GPIO mode: %s", config.Mode)
	}
	if config.Pull != GpioPullNone && config.Pull != GpioPullUp && config.Pull != GpioPullDown {
		return fmt.Errorf("invalid GPIO pull: %s", config.Pull)
	}

	// Create pin state
	state := GpioState{
		Pin:         config.Pin,
		Mode:        config.Mode,
		Pull:        config.Pull,
		Value:       config.InitialValue,
		LastChange:  uint64(time.Now().UnixNano()),
		ChangeCount: 0,
	}

	gs.configuredPins[config.Pin] = state

	// Record operation
	operation := GpioOperationResult{
		OperationID:   fmt.Sprintf("configure_%d", time.Now().UnixNano()),
		Pin:           config.Pin,
		Operation:     "configure",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: gs.deterministicMode,
	}
	gs.operationHistory = append(gs.operationHistory, operation)

	return nil
}

func (gs *MockGpioService) ReadPin(sessionID, caps string, pin uint32) (bool, error) {
	state, exists := gs.configuredPins[pin]
	if !exists {
		return false, fmt.Errorf("pin %d not configured", pin)
	}

	// Record operation
	operation := GpioOperationResult{
		OperationID:   fmt.Sprintf("read_%d", time.Now().UnixNano()),
		Pin:           pin,
		Operation:     "read",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: gs.deterministicMode,
	}
	gs.operationHistory = append(gs.operationHistory, operation)

	return state.Value, nil
}

func (gs *MockGpioService) WritePin(sessionID, caps string, pin uint32, value bool) error {
	state, exists := gs.configuredPins[pin]
	if !exists {
		return fmt.Errorf("pin %d not configured", pin)
	}
	if state.Mode != GpioModeOutput {
		return fmt.Errorf("pin %d not configured as output", pin)
	}

	// Update pin state
	state.Value = value
	state.LastChange = uint64(time.Now().UnixNano())
	state.ChangeCount++
	gs.configuredPins[pin] = state

	// Record operation
	operation := GpioOperationResult{
		OperationID:   fmt.Sprintf("write_%d", time.Now().UnixNano()),
		Pin:           pin,
		Operation:     "write",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: gs.deterministicMode,
	}
	gs.operationHistory = append(gs.operationHistory, operation)

	return nil
}

func (gs *MockGpioService) TogglePin(sessionID, caps string, pin uint32) (bool, error) {
	state, exists := gs.configuredPins[pin]
	if !exists {
		return false, fmt.Errorf("pin %d not configured", pin)
	}
	if state.Mode != GpioModeOutput {
		return false, fmt.Errorf("pin %d not configured as output", pin)
	}

	// Toggle value
	newValue := !state.Value
	state.Value = newValue
	state.LastChange = uint64(time.Now().UnixNano())
	state.ChangeCount++
	gs.configuredPins[pin] = state

	// Record operation
	operation := GpioOperationResult{
		OperationID:   fmt.Sprintf("toggle_%d", time.Now().UnixNano()),
		Pin:           pin,
		Operation:     "toggle",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: gs.deterministicMode,
	}
	gs.operationHistory = append(gs.operationHistory, operation)

	return newValue, nil
}

func (gs *MockGpioService) ListConfiguredPins() ([]GpioState, error) {
	pins := make([]GpioState, 0, len(gs.configuredPins))
	for _, pin := range gs.configuredPins {
		pins = append(pins, pin)
	}
	return pins, nil
}

func (gs *MockGpioService) GetOperationHistory(limit int) ([]GpioOperationResult, error) {
	operations := gs.operationHistory
	if limit > 0 && limit < len(operations) {
		operations = operations[len(operations)-limit:]
	}
	return operations, nil
}

func (gs *MockGpioService) CreateSnapshot(outputPath string) (string, error) {
	snapshotID := fmt.Sprintf("gpio_snapshot_%d", time.Now().UnixNano())
	
	// TODO: Create actual NGFS snapshot
	fmt.Printf("Creating NGFS snapshot at: %s\n", outputPath)
	
	return snapshotID, nil
}

func (gs *MockGpioService) EnableDeterministicMode() error {
	gs.deterministicMode = true
	return nil
}

func (gs *MockGpioService) DisableDeterministicMode() error {
	gs.deterministicMode = false
	return nil
}

func (gs *MockGpioService) GetDeterministicStatus() (bool, uint64, error) {
	return gs.deterministicMode, gs.tickCounter, nil
}

func (gs *MockGpioService) AdvanceTick() error {
	gs.tickCounter++
	return nil
}

func init() {
	// GPIO configure command flags
	gpioConfigureCmd.Flags().Uint32("pin", 0, "GPIO pin number (required)")
	gpioConfigureCmd.Flags().String("mode", "input", "Pin mode (input, output)")
	gpioConfigureCmd.Flags().String("pull", "none", "Pull configuration (none, up, down)")
	gpioConfigureCmd.Flags().Bool("initial-value", false, "Initial value for output pins")
	gpioConfigureCmd.Flags().Uint32("debounce", 0, "Debounce time in milliseconds")

	// GPIO read command flags
	gpioReadCmd.Flags().Bool("json", false, "Output in JSON format")

	// GPIO toggle command flags
	gpioToggleCmd.Flags().Bool("json", false, "Output in JSON format")

	// GPIO list command flags
	gpioListCmd.Flags().Bool("json", false, "Output in JSON format")

	// GPIO history command flags
	gpioHistoryCmd.Flags().Int("limit", 0, "Maximum number of operations to show (0 for all)")
	gpioHistoryCmd.Flags().Bool("json", false, "Output in JSON format")

	// GPIO snapshot command flags
	gpioSnapshotCmd.Flags().String("out", "", "Output NGFS path")

	// Add subcommands
	gpioCmd.AddCommand(
		gpioConfigureCmd,
		gpioReadCmd,
		gpioWriteCmd,
		gpioToggleCmd,
		gpioListCmd,
		gpioHistoryCmd,
		gpioSnapshotCmd,
		gpioDeterministicCmd,
	)
}
