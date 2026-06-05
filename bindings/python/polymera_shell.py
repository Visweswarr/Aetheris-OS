#!/usr/bin/env python3
"""
Polymera Interactive Shell - State-of-the-Art System Automation Console.
Provides a premium, stylized REPL for scripting and inspecting Polymera OS.
"""

import sys
import os
import json
import code
import traceback
from typing import Any

# ANSI styling helper functions for a premium, harmonious HSL-tailored palette.
PURPLE = "\033[38;2;168;85;247m"
CYAN = "\033[38;2;6;182;212m"
GREEN = "\033[38;2;34;197;94m"
YELLOW = "\033[38;2;234;179;8m"
RED = "\033[38;2;239;68;68m"
GREY = "\033[38;2;156;163;175m"
BOLD = "\033[1m"
RESET = "\033[0m"

def print_banner(mode: str):
    """Prints a premium, gorgeous dark-mode dashboard banner."""
    banner = f"""
{PURPLE}╔═══════════════════════════════════════════════════════════════════════════╗
║  {CYAN}⚡ POLYMERA OS AUTOMATION REPL{PURPLE}                                            ║
║  {GREY}State-of-the-Art Polyglot System Console v1.5.0{PURPLE}                          ║
╚═══════════════════════════════════════════════════════════════════════════╝{RESET}
{BOLD}System Status:{RESET}
  {GREY}●{RESET} Host Environment:  {CYAN}{sys.platform.capitalize()}{RESET} (Python {sys.version.split()[0]})
  {GREY}●{RESET} SDK Integration:   {GREEN if "Native" in mode else YELLOW}{mode}{RESET}
  {GREY}●{RESET} Active Capabilities: {GREEN}fs.read{RESET}, {GREEN}fs.write{RESET}, {GREEN}ai.summarize{RESET}, {GREEN}system.inspect{RESET}
  {GREY}●{RESET} Core IPC Gateway:  {CYAN}aetheris-intent-bus (v2.1){RESET}

{GREY}Type {BOLD}.help{RESET}{GREY} to view custom command hooks, or standard Python commands to execute.{RESET}
"""
    print(banner)

def print_help():
    """Prints beautifully aligned help instructions."""
    help_text = f"""
{PURPLE}╭───────────────── {CYAN}POLYMERA SHELL COMMANDS{PURPLE} ─────────────────╮
│ {BOLD}.help{RESET}          Show this gorgeous help instruction card        │
│ {BOLD}.sysinfo{RESET}       Display current system status and active services│
│ {BOLD}.tools{RESET}         List all registered AI core tools and components │
│ {BOLD}.plan "<goal>"{RESET}  Trigger AI Core Planner to execute a goal       │
│ {BOLD}.exit{RESET}          Cleanly exit the interactive automation shell   │
├───────────────────────────────────────────────────────────┤
│ {BOLD}SDK API Preloaded Context:{RESET}                                 │
│   {CYAN}plan(goal){RESET}       - Create and run an AI core plan             │
│   {CYAN}call_tool(t, ...){RESET}- Invoke an AI core tool directly            │
│   {CYAN}memory(k, v=None){RESET}- Put or get data from intent-bus memory     │
│   {CYAN}Service(name){RESET}     - Interface with any OS system service       │
╰───────────────────────────────────────────────────────────╯
"""
    print(help_text)

# Try loading real SDK, fallback to high-quality interactive mock if native library isn't compiled.
SDK_MODE = "Mock SDK Emulation"
_mock_memory = {
    "sys.boot_target": "green",
    "security.policy": "restrictive",
}

try:
    from polymera import Service, call_tool, memory, plan
    # Test if native module loads correctly
    try:
        # Just calling the function helper check to verify native linkage
        import polymera_native
        SDK_MODE = "Native SDK (Production-Grade)"
    except Exception:
        SDK_MODE = "Mock SDK (Native Binding Missing)"
        raise ImportError()
except Exception:
    # High-quality mock implementation for offline validation
    class Service:
        def __init__(self, name: str):
            self.name = name
        def call(self, method: str, **parameters: Any) -> dict[str, Any]:
            print(f"\n{CYAN}┌─── [IPC Call to Service: {self.name}] ───────────────────────────────{RESET}")
            print(f"{GREY}│ Method:     {method}{RESET}")
            print(f"{GREY}│ Parameters: {json.dumps(parameters)}{RESET}")
            print(f"{CYAN}└───────────────────────────────────────────────────────────────────┘{RESET}")
            return {"status": "ok", "service": self.name, "method": method, "parameters": parameters}

    def plan(goal: str, *, session_id: str | None = None, user_id: str | None = None) -> dict[str, Any]:
        print(f"\n{PURPLE}┌─── [AI Core Planner Initializing] ─────────────────────────────────{RESET}")
        print(f"{GREY}│ Goal:       \"{goal}\"{RESET}")
        print(f"{GREY}│ Session ID: {session_id or 'default-session'}{RESET}")
        print(f"{PURPLE}├─ [Execution Pipeline] ─────────────────────────────────────────────{RESET}")
        
        # Emulate logical step execution
        if "summarize" in goal.lower() or "log" in goal.lower():
            print(f"{GREY}│ {GREEN}✓{RESET} Step 1: fs.read -> Loaded log file data (843 bytes){RESET}")
            print(f"{GREY}│ {GREEN}✓{RESET} Step 2: log_summarizer -> Ran WASM sandbox component successfully{RESET}")
            res = {
                "status": "success",
                "goal": goal,
                "steps_executed": 2,
                "result": "5 lines: 2 errors, 1 warnings, 2 info. Top terms: started, high, connection"
            }
        else:
            print(f"{GREY}│ {GREEN}✓{RESET} Step 1: system.inspect -> Inspected service registry{RESET}")
            res = {
                "status": "success",
                "goal": goal,
                "steps_executed": 1,
                "result": f"Inspect results for goal: {goal}"
            }
            
        print(f"{GREY}│ Status:     {GREEN}SUCCESS{RESET}")
        print(f"{GREY}│ Result:     {res['result']}{RESET}")
        print(f"{PURPLE}└───────────────────────────────────────────────────────────────────┘{RESET}")
        return res

    def call_tool(tool_name: str, **parameters: Any) -> dict[str, Any]:
        print(f"\n{CYAN}┌─── [Tool Execution: {tool_name}] ───────────────────────────────{RESET}")
        print(f"{GREY}│ Params:     {json.dumps(parameters)}{RESET}")
        
        if tool_name == "log_summarizer":
            res = {
                "tool_id": "log_summarizer",
                "success": True,
                "result": {
                    "source_path": parameters.get("path", "data/sample.log"),
                    "mode": "wasm-component",
                    "line_count": 5,
                    "error_count": 2,
                    "warning_count": 1,
                    "info_count": 2,
                    "top_terms": ["started", "high", "connection"],
                    "summary": "5 lines: 2 errors, 1 warnings, 2 info. Top terms: started, high, connection"
                }
            }
        elif tool_name == "memory.put":
            _mock_memory[parameters.get("key")] = parameters.get("value")
            res = {"success": True, "key": parameters.get("key"), "value": parameters.get("value")}
        elif tool_name == "memory.get":
            val = _mock_memory.get(parameters.get("key"))
            res = {"success": True, "key": parameters.get("key"), "value": val}
        else:
            res = {"success": True, "msg": f"Mock executed {tool_name} successfully."}
            
        print(f"{GREY}│ Response:   {GREEN if res.get('success') else RED}{json.dumps(res)}{RESET}")
        print(f"{CYAN}└───────────────────────────────────────────────────────────────────┘{RESET}")
        return res

    def memory(key: str, value: Any | None = None) -> dict[str, Any]:
        method = "memory.put" if value is not None else "memory.get"
        return call_tool(method, key=key, value=value)

class PolymeraInteractiveConsole(code.InteractiveConsole):
    """Custom interactive console with support for dot-commands and visual error containment."""
    
    def __init__(self, locals_dict: dict):
        super().__init__(locals=locals_dict)
        
    def raw_input(self, prompt: str = "") -> str:
        # Beautiful prompt styling using HSL CYAN color
        custom_prompt = f"{CYAN}polymera{PURPLE}❯{RESET} "
        return super().raw_input(custom_prompt)
        
    def runsource(self, source: str, filename: str = "<input>", symbol: str = "single") -> bool:
        cmd = source.strip()
        if not cmd:
            return False
            
        # Intercept custom shell commands
        if cmd.startswith("."):
            try:
                self.execute_custom_command(cmd)
            except Exception as e:
                self.print_boxed_error(f"Command Error: {e}", traceback.format_exc())
            return False
            
        # Fall back to standard python execution, catching and wrapping errors in stylized layout
        try:
            # We override compile and run to catch stdout and capture visual feedback
            return super().runsource(source, filename, symbol)
        except Exception as e:
            self.print_boxed_error(f"Execution Exception: {type(e).__name__}", traceback.format_exc())
            return False

    def execute_custom_command(self, cmd: str):
        """Processes dot-commands with customized premium visual layouts."""
        tokens = cmd.split(maxsplit=1)
        base_cmd = tokens[0]
        arg = tokens[1] if len(tokens) > 1 else ""
        
        if base_cmd == ".help":
            print_help()
        elif base_cmd == ".exit":
            print(f"\n{PURPLE}Leaving Polymera Shell. Restoring standard system state...{RESET}")
            sys.exit(0)
        elif base_cmd == ".sysinfo":
            print(f"\n{CYAN}┌─── [System Information] ──────────────────────────────────────────{RESET}")
            print(f"{GREY}│ IPC Gateway:   Aetheris Intent-Bus Active{RESET}")
            print(f"{GREY}│ SDK Version:   1.5.0-polyglot-stable{RESET}")
            print(f"{GREY}│ Security Ring: Ring-3 Sandbox Mode{RESET}")
            print(f"{GREY}│ Active Envs:   Rust Engine, Python REPL, WASM Runtime{RESET}")
            print(f"{CYAN}└───────────────────────────────────────────────────────────────────┘{RESET}")
        elif base_cmd == ".tools":
            print(f"\n{PURPLE}┌─── [Registered AI Core Tools] ────────────────────────────────────{RESET}")
            tools = [
                ("log_summarizer", "WASM Component", "Capability-gated deterministic log summarizer"),
                ("browser_summarize", "Builtin", "Local web page text summarizer"),
                ("browser_classify_page", "Builtin", "Deterministic local URL risk classifier"),
                ("local_llm_summarizer", "External Service", "Fail-closed Ollama integration"),
            ]
            for t_name, t_type, t_desc in tools:
                print(f"{GREY}│ {CYAN}{t_name:<23}{PURPLE}[{t_type}]{RESET} {t_desc}{RESET}")
            print(f"{PURPLE}└───────────────────────────────────────────────────────────────────┘{RESET}")
        elif base_cmd == ".plan":
            if not arg:
                raise ValueError("Goal argument is required. Usage: .plan \"<your-goal>\"")
            # Strip quotes if present
            goal = arg.strip("\"'")
            plan(goal)
        else:
            raise ValueError(f"Unknown shell command: {base_cmd}. Type .help to view valid options.")

    def print_boxed_error(self, title: str, trace: str):
        """Displays error details in a premium customized HSL warning box."""
        print(f"\n{RED}┌─── [⚠️ {title}] ──────────────────────────────────────────────────{RESET}")
        for line in trace.strip().split("\n")[-4:]:  # Show last few lines of traceback
            print(f"{RED}│{RESET} {GREY}{line}{RESET}")
        print(f"{RED}└───────────────────────────────────────────────────────────────────┘{RESET}")

def main():
    # Setup interactive context
    context = {
        "Service": Service,
        "call_tool": call_tool,
        "memory": memory,
        "plan": plan,
        "sys": sys,
        "os": os,
        "json": json,
    }
    
    print_banner(SDK_MODE)
    
    # Instantiate custom console and start REPL loop
    console = PolymeraInteractiveConsole(locals_dict=context)
    console.interact(banner="", exitmsg="")

if __name__ == "__main__":
    main()
