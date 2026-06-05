//go:build windows
// +build windows

// AICore CLI (Go) - minimal wrapper around C FFI
package main

/*
#cgo CFLAGS: -I../c
#cgo LDFLAGS: -laicore
#include "../c/aicore.h"
*/
import "C"
import (
    "bufio"
    "encoding/json"
    "flag"
    "fmt"
    "os"
    "path/filepath"
)

type PlanResult struct {
    Plan            any     `json:"plan"`
    Success         bool    `json:"success"`
    GenerationTime  uint64  `json:"generation_time_us"`
    Error           *string `json:"error"`
}

type submitJSON struct {
    SnapshotID string `json:"snapshot_id"`
    PlanPath   string `json:"plan_path"`
}

// FFI additions
// int aicore_snapshot(unsigned char* out_id_buf, unsigned long long out_len);
// int aicore_replay(const char* snapshot_id);
//export aicore_snapshot
func aicore_snapshot(out_id_buf *C.uchar, out_len C.ulonglong) C.int
//export aicore_replay
func aicore_replay(id *C.char) C.int

func cmdInit() int {
    rc := int(C.aicore_init_default())
    if rc != 0 {
        fmt.Fprintf(os.Stderr, "init failed: %d\n", rc)
    }
    return rc
}

func cmdSubmit(goal string, ngfsRoot string, dumpPlan bool, jsonOut bool) error {
    // Allocate 64KB output buffer
    // TODO: Make this dynamic based on a size query if possible in future C API versions
    out := make([]byte, 65536)
    rc := int(C.aicore_submit_goal_json(C.CString(fmt.Sprintf("\"%s\"", goal)), (*C.uchar)(&out[0]), C.ulonglong(len(out))))
    if rc != 0 {
        return fmt.Errorf("submit failed with code: %d", rc)
    }
    var pr PlanResult
    if err := json.Unmarshal(out, &pr); err == nil {
        // After success, capture snapshot id
        idBuf := make([]byte, 128)
        if int(C.aicore_snapshot((*C.uchar)(&idBuf[0]), C.ulonglong(len(idBuf)))) == 0 {
            snapID := string(idBuf)
            // Trim trailing zeros
            for i := len(snapID)-1; i>=0; i-- { if snapID[i] == 0 { snapID = snapID[:i] } else { break } }
            // Write plan JSON to <ngfsRoot>/plans/<id>.json
            if ngfsRoot == "" { ngfsRoot = "./data/ngfs" }
            planDir := filepath.Join(ngfsRoot, "plans")
            if err := os.MkdirAll(planDir, 0o755); err != nil {
                return fmt.Errorf("failed to create plan dir: %w", err)
            }
            planPath := filepath.Join(planDir, fmt.Sprintf("%s.json", snapID))
            enc, _ := json.MarshalIndent(pr, "", "  ")
            if err := os.WriteFile(planPath, enc, 0o644); err != nil {
                return fmt.Errorf("failed to write plan file: %w", err)
            }
            if jsonOut {
                // Machine-readable single-object JSON
                obj := submitJSON{SnapshotID: snapID, PlanPath: planPath}
                if b, err := json.Marshal(obj); err == nil {
                    fmt.Println(string(b))
                } else {
                    return fmt.Errorf("json marshal error: %w", err)
                }
            } else if dumpPlan {
                // Human-friendly: print full plan JSON
                fmt.Println(string(enc))
            } else {
                // Default: quiet, just the snapshot id line
                fmt.Printf("snapshot_id=%s\n", snapID)
            }
        } else {
            return fmt.Errorf("snapshot failed")
        }
    } else {
        // Fallback raw (only when not explicitly requesting --json)
        if jsonOut {
            return fmt.Errorf("unexpected non-JSON plan output")
        } else {
            fmt.Println(string(out))
        }
    }
    return nil
}

func cmdReplay(id, ngfsRoot string) error {
    // Set NGFS_ROOT environment for the core
    if ngfsRoot != "" { os.Setenv("NGFS_ROOT", ngfsRoot) }
    rc := int(C.aicore_replay(C.CString(id)))
    if rc != 0 {
        return fmt.Errorf("replay failed with code: %d", rc)
    }
    fmt.Println("replay ok")
    return nil
}

func repl(ngfsRoot, captoken string, dumpPlan, jsonOut bool) error {
    if ngfsRoot != "" { os.Setenv("NGFS_ROOT", ngfsRoot) }
    if captoken != "" { os.Setenv("AICORE_CAPTOKEN", captoken) }
    if rc := cmdInit(); rc != 0 { return fmt.Errorf("init failed with code: %d", rc) }
    fmt.Println("AICore REPL — enter goals, Ctrl+D to exit")
    scanner := bufio.NewScanner(os.Stdin)
    for {
        fmt.Print("> ")
        if !scanner.Scan() { break }
        line := scanner.Text()
        if len(line) == 0 { continue }
        // Submit each line as a goal
        if err := cmdSubmit(line, ngfsRoot, dumpPlan, jsonOut); err != nil {
            fmt.Fprintf(os.Stderr, "Error: %v\n", err)
        }
    }
    if err := scanner.Err(); err != nil {
        return fmt.Errorf("repl error: %w", err)
    }
    return nil
}

func run() error {
    // Global flags
    ngfsRoot := flag.String("ngfs-root", "./data/ngfs", "NGFS root path")
    replayID := flag.String("replay", "", "Replay snapshot id")
    dumpPlan := flag.Bool("dump-plan", false, "Dump last plan JSON")
    jsonOut := flag.Bool("json", false, "Print JSON {snapshot_id, plan_path} and nothing else")
    interactive := flag.Bool("interactive", false, "Run interactive REPL for goals")
    captoken := flag.String("captoken", "", "Path to CapToken to attach to intents")

    flag.Parse()
    args := flag.Args()

    if *interactive {
        return repl(*ngfsRoot, *captoken, *dumpPlan, *jsonOut)
    }

    if len(args) < 1 {
        return fmt.Errorf("Usage: aicore <init|submit|replay> [goal] [--ngfs-root PATH] [--replay ID] [--dump-plan] [--json] [--interactive] [--captoken FILE]")
    }
    switch args[0] {
    case "init":
        if *ngfsRoot != "" { os.Setenv("NGFS_ROOT", *ngfsRoot) }
        if *captoken != "" { os.Setenv("AICORE_CAPTOKEN", *captoken) }
        if rc := cmdInit(); rc != 0 {
            return fmt.Errorf("init failed with code: %d", rc)
        }
        return nil
    case "submit":
        if len(args) < 2 {
            return fmt.Errorf("missing goal")
        }
        if *ngfsRoot != "" { os.Setenv("NGFS_ROOT", *ngfsRoot) }
        if *captoken != "" { os.Setenv("AICORE_CAPTOKEN", *captoken) }
        return cmdSubmit(args[1], *ngfsRoot, *dumpPlan, *jsonOut)
    case "replay":
        if *replayID == "" {
            return fmt.Errorf("--replay <id> required")
        }
        return cmdReplay(*replayID, *ngfsRoot)
    default:
        return fmt.Errorf("unknown command: %s", args[0])
    }
}

func main() {
    if err := run(); err != nil {
        fmt.Fprintf(os.Stderr, "%v\n", err)
        os.Exit(1)
    }
}
}
