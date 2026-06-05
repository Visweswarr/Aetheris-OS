# Message builder utilities for AI Core Service
"""
Convenience functions for building AI Core Service messages.
"""

import uuid
import time
from typing import Any, Dict, List, Optional, Union
import ai_core_pb2
from cbor_utils import CborUtils


def create_ping_request(client_id: str, version: str) -> ai_core_pb2.AiCoreMessage:
    """
    Create a ping request message.
    
    Args:
        client_id: Client identifier
        version: Client version
        
    Returns:
        AiCoreMessage with ping request
    """
    ping_request = ai_core_pb2.PingRequest(
        client_id=client_id,
        version=version
    )
    
    return ai_core_pb2.AiCoreMessage(
        ping_request=ping_request,
        message_id=str(uuid.uuid4()),
        timestamp=int(time.time()),
        session_id=str(uuid.uuid4())
    )


def create_chat_request(
    prompt: str,
    config: Optional[ai_core_pb2.ChatConfig] = None,
    conversation_id: Optional[str] = None,
    user_id: Optional[str] = None,
    stream: bool = False,
    enable_function_calling: bool = False,
    allowed_functions: Optional[List[str]] = None
) -> ai_core_pb2.AiCoreMessage:
    """
    Create a chat request message.
    
    Args:
        prompt: The chat prompt
        config: Chat configuration (optional)
        conversation_id: Conversation identifier (optional)
        user_id: User identifier (optional)
        stream: Whether to stream the response
        enable_function_calling: Whether to enable function calling
        allowed_functions: List of allowed function names (optional)
        
    Returns:
        AiCoreMessage with chat request
    """
    chat_request = ai_core_pb2.ChatRequest(
        prompt=prompt,
        config=config,
        context=[],
        stream=stream,
        conversation_id=conversation_id or "",
        user_id=user_id or "",
        metadata={},
        attachments=[],
        enable_function_calling=enable_function_calling,
        allowed_functions=allowed_functions or []
    )
    
    return ai_core_pb2.AiCoreMessage(
        chat_request=chat_request,
        message_id=str(uuid.uuid4()),
        timestamp=int(time.time()),
        session_id=str(uuid.uuid4())
    )


def create_tool_call_request(
    tool_name: str,
    parameters: Dict[str, str],
    context: Optional[str] = None,
    cbor_payload: Optional[bytes] = None,
    call_id: Optional[str] = None,
    session_id: Optional[str] = None,
    timeout_seconds: int = 30,
    async_: bool = False
) -> ai_core_pb2.AiCoreMessage:
    """
    Create a tool call request message.
    
    Args:
        tool_name: Name of the tool to call
        parameters: Tool parameters
        context: Additional context (optional)
        cbor_payload: CBOR-encoded structured data (optional)
        call_id: Call identifier (optional)
        session_id: Session identifier (optional)
        timeout_seconds: Timeout in seconds
        async_: Whether to execute asynchronously
        
    Returns:
        AiCoreMessage with tool call request
    """
    tool_call_request = ai_core_pb2.ToolCallRequest(
        tool_name=tool_name,
        parameters=parameters,
        context=context or "",
        cbor_payload=cbor_payload or b"",
        call_id=call_id or str(uuid.uuid4()),
        session_id=session_id or "",
        metadata={},
        timeout_seconds=timeout_seconds,
        async_=async_
    )
    
    return ai_core_pb2.AiCoreMessage(
        tool_call_request=tool_call_request,
        message_id=str(uuid.uuid4()),
        timestamp=int(time.time()),
        session_id=session_id or str(uuid.uuid4())
    )


def create_function_result(
    function_name: str,
    call_id: str,
    success: bool,
    result: str = "",
    error_message: str = "",
    exit_code: int = 0,
    cbor_payload: Optional[bytes] = None,
    execution_time_ms: int = 0,
    memory_used_mb: int = 0
) -> ai_core_pb2.AiCoreMessage:
    """
    Create a function result message.
    
    Args:
        function_name: Name of the function
        call_id: Call identifier
        success: Whether the function succeeded
        result: Function result (optional)
        error_message: Error message if failed (optional)
        exit_code: Exit code
        cbor_payload: CBOR-encoded structured result (optional)
        execution_time_ms: Execution time in milliseconds
        memory_used_mb: Memory used in MB
        
    Returns:
        AiCoreMessage with function result
    """
    function_result = ai_core_pb2.FunctionResult(
        function_name=function_name,
        call_id=call_id,
        success=success,
        result=result,
        cbor_payload=cbor_payload or b"",
        error_message=error_message,
        exit_code=exit_code,
        metadata={},
        execution_time_ms=execution_time_ms,
        memory_used_mb=memory_used_mb
    )
    
    return ai_core_pb2.AiCoreMessage(
        function_result=function_result,
        message_id=str(uuid.uuid4()),
        timestamp=int(time.time()),
        session_id=str(uuid.uuid4())
    )


def create_error_envelope(
    code: ai_core_pb2.ErrorCode,
    message: str,
    details: Optional[str] = None,
    error_id: Optional[str] = None,
    component: Optional[str] = None,
    operation: Optional[str] = None,
    context: Optional[Dict[str, str]] = None,
    stack_trace: Optional[List[str]] = None,
    suggestion: Optional[str] = None,
    retryable: bool = False,
    retry_after_seconds: int = 0
) -> ai_core_pb2.AiCoreMessage:
    """
    Create an error envelope message.
    
    Args:
        code: Error code
        message: Error message
        details: Additional error details (optional)
        error_id: Error identifier (optional)
        component: Component that generated the error (optional)
        operation: Operation being performed (optional)
        context: Additional context (optional)
        stack_trace: Stack trace (optional)
        suggestion: Suggested remediation (optional)
        retryable: Whether the operation can be retried
        retry_after_seconds: Suggested retry delay
        
    Returns:
        AiCoreMessage with error envelope
    """
    error_envelope = ai_core_pb2.ErrorEnvelope(
        code=code,
        message=message,
        details=details or "",
        timestamp=int(time.time()),
        error_id=error_id or "",
        component=component or "",
        operation=operation or "",
        context=context or {},
        stack_trace=stack_trace or [],
        suggestion=suggestion or "",
        retryable=retryable,
        retry_after_seconds=retry_after_seconds
    )
    
    return ai_core_pb2.AiCoreMessage(
        error_envelope=error_envelope,
        message_id=str(uuid.uuid4()),
        timestamp=int(time.time()),
        session_id=str(uuid.uuid4())
    )


def create_cap_token(
    token_id: str,
    capability: str,
    expires_at: int,
    signature: bytes,
    issuer: str
) -> ai_core_pb2.CapToken:
    """
    Create a capability token.
    
    Args:
        token_id: Token identifier
        capability: Capability string
        expires_at: Expiration timestamp
        signature: Token signature
        issuer: Token issuer
        
    Returns:
        CapToken
    """
    return ai_core_pb2.CapToken(
        token_id=token_id,
        capability=capability,
        expires_at=expires_at,
        signature=signature,
        issuer=issuer
    )


def create_chat_config(
    model: str,
    temperature: float = 0.7,
    max_tokens: int = 1000,
    top_p: float = 0.9,
    top_k: int = 40,
    enable_tools: bool = False,
    allowed_tools: Optional[List[str]] = None
) -> ai_core_pb2.ChatConfig:
    """
    Create a chat configuration.
    
    Args:
        model: Model name
        temperature: Sampling temperature
        max_tokens: Maximum tokens to generate
        top_p: Top-p sampling parameter
        top_k: Top-k sampling parameter
        enable_tools: Whether to enable tools
        allowed_tools: List of allowed tools (optional)
        
    Returns:
        ChatConfig
    """
    return ai_core_pb2.ChatConfig(
        model=model,
        temperature=temperature,
        max_tokens=max_tokens,
        top_p=top_p,
        top_k=top_k,
        enable_tools=enable_tools,
        allowed_tools=allowed_tools or []
    )


def create_message_context(
    role: str,
    content: str,
    timestamp: Optional[int] = None,
    metadata: Optional[Dict[str, str]] = None
) -> ai_core_pb2.MessageContext:
    """
    Create a message context.
    
    Args:
        role: Message role (user, assistant, system, tool)
        content: Message content
        timestamp: Message timestamp (optional)
        metadata: Additional metadata (optional)
        
    Returns:
        MessageContext
    """
    return ai_core_pb2.MessageContext(
        role=role,
        content=content,
        timestamp=timestamp or int(time.time()),
        metadata=metadata or {}
    )


def create_tool_call(
    tool_name: str,
    parameters: Dict[str, str],
    call_id: Optional[str] = None,
    function_name: Optional[str] = None,
    cbor_payload: Optional[bytes] = None,
    timeout_seconds: int = 30,
    async_: bool = False
) -> ai_core_pb2.ToolCall:
    """
    Create a tool call.
    
    Args:
        tool_name: Name of the tool
        parameters: Tool parameters
        call_id: Call identifier (optional)
        function_name: Specific function name (optional)
        cbor_payload: CBOR-encoded parameters (optional)
        timeout_seconds: Timeout in seconds
        async_: Whether to execute asynchronously
        
    Returns:
        ToolCall
    """
    return ai_core_pb2.ToolCall(
        tool_name=tool_name,
        parameters=parameters,
        call_id=call_id or str(uuid.uuid4()),
        function_name=function_name or "",
        cbor_payload=cbor_payload or b"",
        metadata={},
        timeout_seconds=timeout_seconds,
        async_=async_
    )
