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

// Actuator type enumeration
type ActuatorType string

const (
	ActuatorTypePWM      ActuatorType = "pwm"
	ActuatorTypeRelay    ActuatorType = "relay"
	ActuatorTypeMotor    ActuatorType = "motor"
	ActuatorTypeLED      ActuatorType = "led"
	ActuatorTypeServo    ActuatorType = "servo"
	ActuatorTypeStepper  ActuatorType = "stepper"
	ActuatorTypeSolenoid ActuatorType = "solenoid"
	ActuatorTypeValve    ActuatorType = "valve"
)

// Actuator configuration
type ActuatorConfig struct {
	Name              string        `json:"name"`
	Type              ActuatorType  `json:"type"`
	Pin               uint32        `json:"pin"`
	MinValue          float64       `json:"min_value"`
	MaxValue          float64       `json:"max_value"`
	DefaultValue      float64       `json:"default_value"`
	Frequency         uint32        `json:"frequency"`
	Resolution        uint32        `json:"resolution"`
	EnableSafetyLimits bool         `json:"enable_safety_limits"`
	SafetyMin         float64       `json:"safety_min"`
	SafetyMax         float64       `json:"safety_max"`
	RampTimeMs        uint32        `json:"ramp_time_ms"`
	EnableRamping     bool          `json:"enable_ramping"`
}

// Actuator state
type ActuatorState struct {
	Name              string        `json:"name"`
	Type              ActuatorType  `json:"type"`
	Configured        bool          `json:"configured"`
	Enabled           bool          `json:"enabled"`
	CurrentValue      float64       `json:"current_value"`
	TargetValue       float64       `json:"target_value"`
	LastUpdate        uint64        `json:"last_update"`
	UpdateCount       uint64        `json:"update_count"`
	SafetyEnabled     bool          `json:"safety_enabled"`
	SafetyMin         float64       `json:"safety_min"`
	SafetyMax         float64       `json:"safety_max"`
	RampingEnabled    bool          `json:"ramping_enabled"`
	RampTimeMs        uint32        `json:"ramp_time_ms"`
	Deterministic     bool          `json:"deterministic"`
}

// Actuator operation result
type ActuatorOperationResult struct {
	OperationID   string `json:"operation_id"`
	ActuatorName  string `json:"actuator_name"`
	Operation     string `json:"operation"`
	Result        bool   `json:"result"`
	Timestamp     uint64 `json:"timestamp"`
	Deterministic bool   `json:"deterministic"`
}

// Actuator pattern step
type ActuatorPatternStep struct {
	Value       float64 `json:"value"`
	DurationMs  uint32  `json:"duration_ms"`
}

// Actuator pattern
type ActuatorPattern struct {
	Name       string                `json:"name"`
	StepCount  uint32                `json:"step_count"`
	Steps      []ActuatorPatternStep `json:"steps"`
	Loop       bool                  `json:"loop"`
	LoopCount  uint32                `json:"loop_count"`
}

// Mock actuator service for demonstration
type MockActuatorService struct {
	configuredActuators map[string]ActuatorState
	operationHistory    []ActuatorOperationResult
	tickCounter         uint64
	deterministicMode   bool
}

var actuatorService = &MockActuatorService{
	configuredActuators: make(map[string]ActuatorState),
	operationHistory:    make([]ActuatorOperationResult, 0),
	tickCounter:         0,
	deterministicMode:   false,
}

// Actuator commands
var actuatorCmd = &cobra.Command{
	Use:   "actuator",
	Short: "Manage actuator operations",
	Long:  "Manage actuator operations, including PWM outputs, relays, motor control, LEDs, and deterministic state management.",
}

var actuatorConfigureCmd = &cobra.Command{
	Use:   "configure",
	Short: "Configure actuator",
	Long:  "Configure an actuator with type, pin, value range, and safety settings.",
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		name, _ := cmd.Flags().GetString("name")
		typeStr, _ := cmd.Flags().GetString("type")
		pin, _ := cmd.Flags().GetUint32("pin")
		minValue, _ := cmd.Flags().GetFloat64("min-value")
		maxValue, _ := cmd.Flags().GetFloat64("max-value")
		defaultValue, _ := cmd.Flags().GetFloat64("default-value")
		frequency, _ := cmd.Flags().GetUint32("frequency")
		resolution, _ := cmd.Flags().GetUint32("resolution")
		enableSafetyLimits, _ := cmd.Flags().GetBool("enable-safety-limits")
		safetyMin, _ := cmd.Flags().GetFloat64("safety-min")
		safetyMax, _ := cmd.Flags().GetFloat64("safety-max")
		rampTimeMs, _ := cmd.Flags().GetUint32("ramp-time")
		enableRamping, _ := cmd.Flags().GetBool("enable-ramping")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:actuator.configure"
		}

		if name == "" {
			fmt.Printf("Error: actuator name is required\n")
			return
		}

		actuatorType := ActuatorType(typeStr)
		if actuatorType == "" {
			actuatorType = ActuatorTypePWM
		}

		config := ActuatorConfig{
			Name:               name,
			Type:               actuatorType,
			Pin:                pin,
			MinValue:           minValue,
			MaxValue:           maxValue,
			DefaultValue:       defaultValue,
			Frequency:          frequency,
			Resolution:         resolution,
			EnableSafetyLimits: enableSafetyLimits,
			SafetyMin:          safetyMin,
			SafetyMax:          safetyMax,
			RampTimeMs:         rampTimeMs,
			EnableRamping:      enableRamping,
		}

		err := actuatorService.ConfigureActuator(sessionID, caps, config)
		if err != nil {
			fmt.Printf("Error configuring actuator: %v\n", err)
			return
		}

		fmt.Printf("Actuator configured successfully\n")
		fmt.Printf("Name: %s\n", config.Name)
		fmt.Printf("Type: %s\n", config.Type)
		fmt.Printf("Pin: %d\n", config.Pin)
		fmt.Printf("Value range: %.3f - %.3f\n", config.MinValue, config.MaxValue)
		fmt.Printf("Default value: %.3f\n", config.DefaultValue)
		fmt.Printf("Frequency: %d Hz\n", config.Frequency)
		fmt.Printf("Resolution: %d bits\n", config.Resolution)
		fmt.Printf("Safety limits: %v\n", config.EnableSafetyLimits)
		if config.EnableSafetyLimits {
			fmt.Printf("  Safety range: %.3f - %.3f\n", config.SafetyMin, config.SafetyMax)
		}
		fmt.Printf("Ramping: %v\n", config.EnableRamping)
		if config.EnableRamping {
			fmt.Printf("  Ramp time: %d ms\n", config.RampTimeMs)
		}
	},
}

var actuatorEnableCmd = &cobra.Command{
	Use:   "enable [name]",
	Short: "Enable actuator",
	Long:  "Enable a configured actuator for control.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:actuator.enable"
		}

		name := args[0]

		err := actuatorService.EnableActuator(sessionID, caps, name)
		if err != nil {
			fmt.Printf("Error enabling actuator: %v\n", err)
			return
		}

		fmt.Printf("Actuator '%s' enabled\n", name)
	},
}

var actuatorDisableCmd = &cobra.Command{
	Use:   "disable [name]",
	Short: "Disable actuator",
	Long:  "Disable an actuator to stop control.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:actuator.disable"
		}

		name := args[0]

		err := actuatorService.DisableActuator(sessionID, caps, name)
		if err != nil {
			fmt.Printf("Error disabling actuator: %v\n", err)
			return
		}

		fmt.Printf("Actuator '%s' disabled\n", name)
	},
}

var actuatorSetCmd = &cobra.Command{
	Use:   "set [name] [value]",
	Short: "Set actuator value",
	Long:  "Set the output value of an enabled actuator.",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:actuator.set"
		}

		name := args[0]
		value, err := strconv.ParseFloat(args[1], 64)
		if err != nil {
			fmt.Printf("Error: invalid value '%s': %v\n", args[1], err)
			return
		}

		err = actuatorService.SetActuatorValue(sessionID, caps, name, value)
		if err != nil {
			fmt.Printf("Error setting actuator value: %v\n", err)
			return
		}

		fmt.Printf("Actuator '%s' set to %.3f\n", name, value)
	},
}

var actuatorGetCmd = &cobra.Command{
	Use:   "get [name]",
	Short: "Get actuator value",
	Long:  "Get the current value of an actuator.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		name := args[0]
		jsonOutput, _ := cmd.Flags().GetBool("json")

		value, err := actuatorService.GetActuatorValue(name)
		if err != nil {
			fmt.Printf("Error getting actuator value: %v\n", err)
			return
		}

		if jsonOutput {
			result := map[string]interface{}{
				"name":  name,
				"value": value,
			}
			jsonData, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			fmt.Printf("Actuator '%s' value: %.3f\n", name, value)
		}
	},
}

var actuatorPatternCmd = &cobra.Command{
	Use:   "pattern [name]",
	Short: "Set actuator pattern",
	Long:  "Set a pattern sequence for an actuator to follow.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")
		patternName, _ := cmd.Flags().GetString("pattern-name")
		stepsStr, _ := cmd.Flags().GetString("steps")
		loop, _ := cmd.Flags().GetBool("loop")
		loopCount, _ := cmd.Flags().GetUint32("loop-count")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:actuator.pattern"
		}

		name := args[0]

		if patternName == "" {
			patternName = fmt.Sprintf("pattern_%d", time.Now().Unix())
		}

		// Parse steps (format: "value1:duration1,value2:duration2,...")
		var steps []ActuatorPatternStep
		if stepsStr != "" {
			stepPairs := strings.Split(stepsStr, ",")
			for _, pair := range stepPairs {
				parts := strings.Split(pair, ":")
				if len(parts) != 2 {
					fmt.Printf("Error: invalid step format '%s'. Use 'value:duration'\n", pair)
					return
				}
				value, err := strconv.ParseFloat(parts[0], 64)
				if err != nil {
					fmt.Printf("Error: invalid step value '%s': %v\n", parts[0], err)
					return
				}
				duration, err := strconv.ParseUint(parts[1], 10, 32)
				if err != nil {
					fmt.Printf("Error: invalid step duration '%s': %v\n", parts[1], err)
					return
				}
				steps = append(steps, ActuatorPatternStep{
					Value:      value,
					DurationMs: uint32(duration),
				})
			}
		}

		pattern := ActuatorPattern{
			Name:       patternName,
			StepCount:  uint32(len(steps)),
			Steps:      steps,
			Loop:       loop,
			LoopCount:  loopCount,
		}

		err := actuatorService.SetActuatorPattern(sessionID, caps, name, pattern)
		if err != nil {
			fmt.Printf("Error setting actuator pattern: %v\n", err)
			return
		}

		fmt.Printf("Actuator pattern set successfully\n")
		fmt.Printf("Actuator: %s\n", name)
		fmt.Printf("Pattern: %s\n", pattern.Name)
		fmt.Printf("Steps: %d\n", pattern.StepCount)
		fmt.Printf("Loop: %v\n", pattern.Loop)
		if pattern.Loop {
			fmt.Printf("Loop count: %d\n", pattern.LoopCount)
		}
	},
}

var actuatorStopPatternCmd = &cobra.Command{
	Use:   "stop-pattern [name]",
	Short: "Stop actuator pattern",
	Long:  "Stop the current pattern execution on an actuator.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		sessionID, _ := cmd.Flags().GetString("session")
		caps, _ := cmd.Flags().GetString("caps")

		if sessionID == "" {
			sessionID = "mock_session_" + fmt.Sprintf("%d", time.Now().Unix())
		}
		if caps == "" {
			caps = "device:actuator.stop-pattern"
		}

		name := args[0]

		err := actuatorService.StopActuatorPattern(sessionID, caps, name)
		if err != nil {
			fmt.Printf("Error stopping actuator pattern: %v\n", err)
			return
		}

		fmt.Printf("Actuator pattern stopped for '%s'\n", name)
	},
}

var actuatorListCmd = &cobra.Command{
	Use:   "list",
	Short: "List configured actuators",
	Long:  "List all configured actuators with their current state.",
	Run: func(cmd *cobra.Command, args []string) {
		jsonOutput, _ := cmd.Flags().GetBool("json")

		actuators, err := actuatorService.ListConfiguredActuators()
		if err != nil {
			fmt.Printf("Error listing actuators: %v\n", err)
			return
		}

		if jsonOutput {
			jsonData, _ := json.MarshalIndent(actuators, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			if len(actuators) == 0 {
				fmt.Println("No actuators configured")
				return
			}

			fmt.Printf("Configured actuators (%d):\n", len(actuators))
			for i, actuator := range actuators {
				fmt.Printf("  %d. %s (%s)\n", i+1, actuator.Name, actuator.Type)
				fmt.Printf("     Configured: %v\n", actuator.Configured)
				fmt.Printf("     Enabled: %v\n", actuator.Enabled)
				fmt.Printf("     Current value: %.3f\n", actuator.CurrentValue)
				fmt.Printf("     Target value: %.3f\n", actuator.TargetValue)
				fmt.Printf("     Update count: %d\n", actuator.UpdateCount)
				fmt.Printf("     Safety limits: %v\n", actuator.SafetyEnabled)
				if actuator.SafetyEnabled {
					fmt.Printf("       Safety range: %.3f - %.3f\n", actuator.SafetyMin, actuator.SafetyMax)
				}
				fmt.Printf("     Ramping: %v\n", actuator.RampingEnabled)
				if actuator.RampingEnabled {
					fmt.Printf("       Ramp time: %d ms\n", actuator.RampTimeMs)
				}
				fmt.Printf("     Deterministic: %v\n", actuator.Deterministic)
				fmt.Println()
			}
		}
	},
}

var actuatorHistoryCmd = &cobra.Command{
	Use:   "history",
	Short: "Show actuator operation history",
	Long:  "Show the history of actuator operations with timestamps and deterministic flags.",
	Run: func(cmd *cobra.Command, args []string) {
		limit, _ := cmd.Flags().GetInt("limit")
		jsonOutput, _ := cmd.Flags().GetBool("json")

		operations, err := actuatorService.GetOperationHistory(limit)
		if err != nil {
			fmt.Printf("Error getting actuator operation history: %v\n", err)
			return
		}

		if jsonOutput {
			jsonData, _ := json.MarshalIndent(operations, "", "  ")
			fmt.Println(string(jsonData))
		} else {
			if len(operations) == 0 {
				fmt.Println("No actuator operations in history")
				return
			}

			fmt.Printf("Actuator operation history (%d operations):\n", len(operations))
			for i, op := range operations {
				fmt.Printf("  %d. %s\n", i+1, op.OperationID)
				fmt.Printf("     Actuator: %s\n", op.ActuatorName)
				fmt.Printf("     Operation: %s\n", op.Operation)
				fmt.Printf("     Result: %v\n", op.Result)
				fmt.Printf("     Timestamp: %d\n", op.Timestamp)
				fmt.Printf("     Deterministic: %v\n", op.Deterministic)
				fmt.Println()
			}
		}
	},
}

var actuatorSnapshotCmd = &cobra.Command{
	Use:   "snapshot",
	Short: "Create actuator snapshot",
	Long:  "Create a deterministic snapshot of actuator state and save to NGFS.",
	Run: func(cmd *cobra.Command, args []string) {
		outputPath, _ := cmd.Flags().GetString("out")

		if outputPath == "" {
			outputPath = fmt.Sprintf("snaps/actuator_%d.ngfs", time.Now().Unix())
		}

		snapshotID, err := actuatorService.CreateSnapshot(outputPath)
		if err != nil {
			fmt.Printf("Error creating actuator snapshot: %v\n", err)
			return
		}

		fmt.Printf("Actuator snapshot created successfully\n")
		fmt.Printf("Snapshot ID: %s\n", snapshotID)
		fmt.Printf("Output path: %s\n", outputPath)
	},
}

var actuatorDeterministicCmd = &cobra.Command{
	Use:   "deterministic [enable|disable|status]",
	Short: "Manage deterministic mode",
	Long:  "Enable, disable, or check the status of deterministic mode for actuator operations.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		action := args[0]

		switch action {
		case "enable":
			err := actuatorService.EnableDeterministicMode()
			if err != nil {
				fmt.Printf("Error enabling deterministic mode: %v\n", err)
				return
			}
			fmt.Println("Deterministic mode enabled")
		case "disable":
			err := actuatorService.DisableDeterministicMode()
			if err != nil {
				fmt.Printf("Error disabling deterministic mode: %v\n", err)
				return
			}
			fmt.Println("Deterministic mode disabled")
		case "status":
			enabled, tickCount, err := actuatorService.GetDeterministicStatus()
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

// Mock actuator service implementation
func (as *MockActuatorService) ConfigureActuator(sessionID, caps string, config ActuatorConfig) error {
	// Validate configuration
	if config.MinValue >= config.MaxValue {
		return fmt.Errorf("min value must be less than max value")
	}
	if config.DefaultValue < config.MinValue || config.DefaultValue > config.MaxValue {
		return fmt.Errorf("default value must be within min/max range")
	}
	if config.EnableSafetyLimits {
		if config.SafetyMin >= config.SafetyMax {
			return fmt.Errorf("safety min must be less than safety max")
		}
		if config.SafetyMin < config.MinValue || config.SafetyMax > config.MaxValue {
			return fmt.Errorf("safety limits must be within min/max range")
		}
	}

	// Create actuator state
	state := ActuatorState{
		Name:               config.Name,
		Type:               config.Type,
		Configured:         true,
		Enabled:            false,
		CurrentValue:       config.DefaultValue,
		TargetValue:        config.DefaultValue,
		LastUpdate:         uint64(time.Now().UnixNano()),
		UpdateCount:        0,
		SafetyEnabled:      config.EnableSafetyLimits,
		SafetyMin:          config.SafetyMin,
		SafetyMax:          config.SafetyMax,
		RampingEnabled:     config.EnableRamping,
		RampTimeMs:         config.RampTimeMs,
		Deterministic:      as.deterministicMode,
	}

	as.configuredActuators[config.Name] = state

	// Record operation
	operation := ActuatorOperationResult{
		OperationID:   fmt.Sprintf("configure_%d", time.Now().UnixNano()),
		ActuatorName:  config.Name,
		Operation:     "configure",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: as.deterministicMode,
	}
	as.operationHistory = append(as.operationHistory, operation)

	return nil
}

func (as *MockActuatorService) EnableActuator(sessionID, caps, name string) error {
	state, exists := as.configuredActuators[name]
	if !exists {
		return fmt.Errorf("actuator '%s' not configured", name)
	}

	state.Enabled = true
	as.configuredActuators[name] = state

	// Record operation
	operation := ActuatorOperationResult{
		OperationID:   fmt.Sprintf("enable_%d", time.Now().UnixNano()),
		ActuatorName:  name,
		Operation:     "enable",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: as.deterministicMode,
	}
	as.operationHistory = append(as.operationHistory, operation)

	return nil
}

func (as *MockActuatorService) DisableActuator(sessionID, caps, name string) error {
	state, exists := as.configuredActuators[name]
	if !exists {
		return fmt.Errorf("actuator '%s' not configured", name)
	}

	state.Enabled = false
	as.configuredActuators[name] = state

	// Record operation
	operation := ActuatorOperationResult{
		OperationID:   fmt.Sprintf("disable_%d", time.Now().UnixNano()),
		ActuatorName:  name,
		Operation:     "disable",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: as.deterministicMode,
	}
	as.operationHistory = append(as.operationHistory, operation)

	return nil
}

func (as *MockActuatorService) SetActuatorValue(sessionID, caps, name string, value float64) error {
	state, exists := as.configuredActuators[name]
	if !exists {
		return fmt.Errorf("actuator '%s' not configured", name)
	}
	if !state.Enabled {
		return fmt.Errorf("actuator '%s' not enabled", name)
	}

	// Check safety limits
	if state.SafetyEnabled && (value < state.SafetyMin || value > state.SafetyMax) {
		return fmt.Errorf("value %.3f violates safety limits (%.3f - %.3f)", value, state.SafetyMin, state.SafetyMax)
	}

	// Update actuator state
	state.CurrentValue = value
	state.TargetValue = value
	state.LastUpdate = uint64(time.Now().UnixNano())
	state.UpdateCount++
	as.configuredActuators[name] = state

	// Record operation
	operation := ActuatorOperationResult{
		OperationID:   fmt.Sprintf("set_%d", time.Now().UnixNano()),
		ActuatorName:  name,
		Operation:     "set_value",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: as.deterministicMode,
	}
	as.operationHistory = append(as.operationHistory, operation)

	return nil
}

func (as *MockActuatorService) GetActuatorValue(name string) (float64, error) {
	state, exists := as.configuredActuators[name]
	if !exists {
		return 0, fmt.Errorf("actuator '%s' not configured", name)
	}

	return state.CurrentValue, nil
}

func (as *MockActuatorService) SetActuatorPattern(sessionID, caps, name string, pattern ActuatorPattern) error {
	state, exists := as.configuredActuators[name]
	if !exists {
		return fmt.Errorf("actuator '%s' not configured", name)
	}
	if !state.Enabled {
		return fmt.Errorf("actuator '%s' not enabled", name)
	}

	// Record operation
	operation := ActuatorOperationResult{
		OperationID:   fmt.Sprintf("pattern_%d", time.Now().UnixNano()),
		ActuatorName:  name,
		Operation:     "set_pattern",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: as.deterministicMode,
	}
	as.operationHistory = append(as.operationHistory, operation)

	// TODO: Implement pattern execution logic
	fmt.Printf("Pattern '%s' set for actuator '%s' (%d steps)\n", pattern.Name, name, pattern.StepCount)

	return nil
}

func (as *MockActuatorService) StopActuatorPattern(sessionID, caps, name string) error {
	state, exists := as.configuredActuators[name]
	if !exists {
		return fmt.Errorf("actuator '%s' not configured", name)
	}

	// Record operation
	operation := ActuatorOperationResult{
		OperationID:   fmt.Sprintf("stop_pattern_%d", time.Now().UnixNano()),
		ActuatorName:  name,
		Operation:     "stop_pattern",
		Result:        true,
		Timestamp:     uint64(time.Now().UnixNano()),
		Deterministic: as.deterministicMode,
	}
	as.operationHistory = append(as.operationHistory, operation)

	// TODO: Implement pattern stop logic
	fmt.Printf("Pattern stopped for actuator '%s'\n", name)

	return nil
}

func (as *MockActuatorService) ListConfiguredActuators() ([]ActuatorState, error) {
	actuators := make([]ActuatorState, 0, len(as.configuredActuators))
	for _, actuator := range as.configuredActuators {
		actuators = append(actuators, actuator)
	}
	return actuators, nil
}

func (as *MockActuatorService) GetOperationHistory(limit int) ([]ActuatorOperationResult, error) {
	operations := as.operationHistory
	if limit > 0 && limit < len(operations) {
		operations = operations[len(operations)-limit:]
	}
	return operations, nil
}

func (as *MockActuatorService) CreateSnapshot(outputPath string) (string, error) {
	snapshotID := fmt.Sprintf("actuator_snapshot_%d", time.Now().UnixNano())
	
	// TODO: Create actual NGFS snapshot
	fmt.Printf("Creating NGFS snapshot at: %s\n", outputPath)
	
	return snapshotID, nil
}

func (as *MockActuatorService) EnableDeterministicMode() error {
	as.deterministicMode = true
	return nil
}

func (as *MockActuatorService) DisableDeterministicMode() error {
	as.deterministicMode = false
	return nil
}

func (as *MockActuatorService) GetDeterministicStatus() (bool, uint64, error) {
	return as.deterministicMode, as.tickCounter, nil
}

func (as *MockActuatorService) AdvanceTick() error {
	as.tickCounter++
	return nil
}

func init() {
	// Actuator configure command flags
	actuatorConfigureCmd.Flags().String("name", "", "Actuator name (required)")
	actuatorConfigureCmd.Flags().String("type", "pwm", "Actuator type (pwm, relay, motor, led, servo, stepper, solenoid, valve)")
	actuatorConfigureCmd.Flags().Uint32("pin", 0, "Control pin number")
	actuatorConfigureCmd.Flags().Float64("min-value", 0.0, "Minimum value")
	actuatorConfigureCmd.Flags().Float64("max-value", 1.0, "Maximum value")
	actuatorConfigureCmd.Flags().Float64("default-value", 0.0, "Default value")
	actuatorConfigureCmd.Flags().Uint32("frequency", 1000, "PWM frequency in Hz")
	actuatorConfigureCmd.Flags().Uint32("resolution", 8, "PWM resolution in bits")
	actuatorConfigureCmd.Flags().Bool("enable-safety-limits", true, "Enable safety limits")
	actuatorConfigureCmd.Flags().Float64("safety-min", 0.0, "Safety minimum value")
	actuatorConfigureCmd.Flags().Float64("safety-max", 1.0, "Safety maximum value")
	actuatorConfigureCmd.Flags().Uint32("ramp-time", 100, "Ramp time in milliseconds")
	actuatorConfigureCmd.Flags().Bool("enable-ramping", false, "Enable value ramping")

	// Actuator get command flags
	actuatorGetCmd.Flags().Bool("json", false, "Output in JSON format")

	// Actuator pattern command flags
	actuatorPatternCmd.Flags().String("pattern-name", "", "Pattern name")
	actuatorPatternCmd.Flags().String("steps", "", "Pattern steps (format: 'value1:duration1,value2:duration2,...')")
	actuatorPatternCmd.Flags().Bool("loop", false, "Loop the pattern")
	actuatorPatternCmd.Flags().Uint32("loop-count", 0, "Number of loops (0 for infinite)")

	// Actuator list command flags
	actuatorListCmd.Flags().Bool("json", false, "Output in JSON format")

	// Actuator history command flags
	actuatorHistoryCmd.Flags().Int("limit", 0, "Maximum number of operations to show (0 for all)")
	actuatorHistoryCmd.Flags().Bool("json", false, "Output in JSON format")

	// Actuator snapshot command flags
	actuatorSnapshotCmd.Flags().String("out", "", "Output NGFS path")

	// Add subcommands
	actuatorCmd.AddCommand(
		actuatorConfigureCmd,
		actuatorEnableCmd,
		actuatorDisableCmd,
		actuatorSetCmd,
		actuatorGetCmd,
		actuatorPatternCmd,
		actuatorStopPatternCmd,
		actuatorListCmd,
		actuatorHistoryCmd,
		actuatorSnapshotCmd,
		actuatorDeterministicCmd,
	)
}
