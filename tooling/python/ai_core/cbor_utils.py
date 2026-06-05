# CBOR utilities for AI Core Service
"""
CBOR utilities for working with structured data in AI Core Service messages.
"""

import json
import uuid
import time
from typing import Any, Dict, Optional, Union
import cbor2


class CborUtils:
    """Utility class for CBOR encoding and decoding."""
    
    @staticmethod
    def encode(obj: Any) -> bytes:
        """
        Encode a Python object to CBOR bytes.
        
        Args:
            obj: The object to encode
            
        Returns:
            CBOR-encoded bytes
            
        Raises:
            ValueError: If encoding fails
        """
        try:
            return cbor2.dumps(obj)
        except Exception as e:
            raise ValueError(f"Failed to encode to CBOR: {e}") from e
    
    @staticmethod
    def decode(data: bytes) -> Any:
        """
        Decode CBOR bytes to a Python object.
        
        Args:
            data: CBOR-encoded bytes
            
        Returns:
            Decoded Python object
            
        Raises:
            ValueError: If decoding fails
        """
        try:
            return cbor2.loads(data)
        except Exception as e:
            raise ValueError(f"Failed to decode from CBOR: {e}") from e
    
    @staticmethod
    def encode_to_json(obj: Any) -> bytes:
        """
        Encode a Python object to JSON bytes (fallback for simple cases).
        
        Args:
            obj: The object to encode
            
        Returns:
            JSON-encoded bytes
        """
        json_str = json.dumps(obj, default=str)
        return json_str.encode('utf-8')
    
    @staticmethod
    def decode_from_json(data: bytes) -> Any:
        """
        Decode JSON bytes to a Python object (fallback for simple cases).
        
        Args:
            data: JSON-encoded bytes
            
        Returns:
            Decoded Python object
        """
        json_str = data.decode('utf-8')
        return json.loads(json_str)


class MessageBuilder:
    """Builder class for creating AI Core Service messages."""
    
    @staticmethod
    def create_ping_request(client_id: str, version: str) -> 'ai_core_pb2.AiCoreMessage':
        """
        Create a ping request message.
        
        Args:
            client_id: Client identifier
            version: Client version
            
        Returns:
            AiCoreMessage with ping request
        """
        import ai_core_pb2
        
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
    
    @staticmethod
    def create_chat_request(
        prompt: str,
        config: Optional['ai_core_pb2.ChatConfig'] = None,
        conversation_id: Optional[str] = None
    ) -> 'ai_core_pb2.AiCoreMessage':
        """
        Create a chat request message.
        
        Args:
            prompt: The chat prompt
            config: Chat configuration (optional)
            conversation_id: Conversation identifier (optional)
            
        Returns:
            AiCoreMessage with chat request
        """
        import ai_core_pb2
        
        chat_request = ai_core_pb2.ChatRequest(
            prompt=prompt,
            config=config,
            context=[],
            stream=False,
            conversation_id=conversation_id or "",
            metadata={},
            attachments=[],
            enable_function_calling=False,
            allowed_functions=[]
        )
        
        return ai_core_pb2.AiCoreMessage(
            chat_request=chat_request,
            message_id=str(uuid.uuid4()),
            timestamp=int(time.time()),
            session_id=str(uuid.uuid4())
        )
    
    @staticmethod
    def create_tool_call_request(
        tool_name: str,
        parameters: Dict[str, str],
        context: Optional[str] = None
    ) -> 'ai_core_pb2.AiCoreMessage':
        """
        Create a tool call request message.
        
        Args:
            tool_name: Name of the tool to call
            parameters: Tool parameters
            context: Additional context (optional)
            
        Returns:
            AiCoreMessage with tool call request
        """
        import ai_core_pb2
        
        tool_call_request = ai_core_pb2.ToolCallRequest(
            tool_name=tool_name,
            parameters=parameters,
            context=context or "",
            metadata={},
            timeout_seconds=30,
            async_=False
        )
        
        return ai_core_pb2.AiCoreMessage(
            tool_call_request=tool_call_request,
            message_id=str(uuid.uuid4()),
            timestamp=int(time.time()),
            session_id=str(uuid.uuid4())
        )
    
    @staticmethod
    def create_error_envelope(
        code: 'ai_core_pb2.ErrorCode',
        message: str,
        details: Optional[str] = None
    ) -> 'ai_core_pb2.AiCoreMessage':
        """
        Create an error envelope message.
        
        Args:
            code: Error code
            message: Error message
            details: Additional error details (optional)
            
        Returns:
            AiCoreMessage with error envelope
        """
        import ai_core_pb2
        
        error_envelope = ai_core_pb2.ErrorEnvelope(
            code=code,
            message=message,
            details=details or "",
            timestamp=int(time.time()),
            context={},
            stack_trace=[],
            retryable=False,
            retry_after_seconds=0
        )
        
        return ai_core_pb2.AiCoreMessage(
            error_envelope=error_envelope,
            message_id=str(uuid.uuid4()),
            timestamp=int(time.time()),
            session_id=str(uuid.uuid4())
        )


class MessageValidator:
    """Validator class for AI Core Service messages."""
    
    @staticmethod
    def validate_message(message: 'ai_core_pb2.AiCoreMessage') -> bool:
        """
        Validate an AI Core message.
        
        Args:
            message: The message to validate
            
        Returns:
            True if valid, False otherwise
        """
        if not message.message_id:
            return False
        
        if message.timestamp <= 0:
            return False
        
        if not message.session_id:
            return False
        
        # Check that exactly one message type is set
        message_types = [
            message.ping_request,
            message.ping_response,
            message.chat_request,
            message.chat_response,
            message.tool_call_request,
            message.tool_call_response,
            message.function_result,
            message.error_envelope,
            message.error_response
        ]
        
        set_types = sum(1 for msg_type in message_types if msg_type is not None)
        if set_types != 1:
            return False
        
        return True
    
    @staticmethod
    def validate_cap_token(token: 'ai_core_pb2.CapToken') -> bool:
        """
        Validate a capability token.
        
        Args:
            token: The token to validate
            
        Returns:
            True if valid, False otherwise
        """
        if not token.token_id:
            return False
        
        if not token.capability:
            return False
        
        if token.expires_at <= 0:
            return False
        
        if not token.issuer:
            return False
        
        return True


# Global instances for convenience
default_cbor_utils = CborUtils()
default_message_builder = MessageBuilder()
default_message_validator = MessageValidator()
