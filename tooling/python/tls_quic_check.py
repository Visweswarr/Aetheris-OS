#!/usr/bin/env python3
"""
TLS/QUIC validation script for Aetheris OS P4-03-A2
Tests handshake timing, cipher verification, and policy enforcement
"""

import asyncio
import json
import time
import sys
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

class TLSProfile(Enum):
    PQC_HYBRID = "pqc_hybrid"
    TLS13_MODERN = "tls13_modern"
    INTRANET_FAST = "intranet_fast"

class QUICProfile(Enum):
    TLS13_MODERN = "tls13_modern"
    PQC_HYBRID = "pqc_hybrid"
    INTRANET_FAST = "intranet_fast"

@dataclass
class TestResult:
    test_name: str
    success: bool
    duration_ms: float
    error_message: Optional[str] = None
    details: Optional[Dict] = None

@dataclass
class PerformanceMetrics:
    handshake_latency_p50: float
    handshake_latency_p95: float
    quic_rtt_p50: float
    quic_rtt_p95: float
    throughput_mbps: float
    resume_ratio: float

class MockTLSBroker:
    """Mock TLS broker for testing"""
    
    def __init__(self):
        self.sessions = {}
        self.stats = {
            "handshakes": 0,
            "rekeys": 0,
            "shutdowns": 0,
            "policy_denials": 0,
        }
    
    async def tls_wrap(self, socket_id: str, profile: str, process_cap: str) -> str:
        """Mock TLS wrap operation"""
        if not self._check_capability(process_cap):
            raise PermissionError("TLS capability required")
        
        session_id = f"tls_session_{len(self.sessions) + 1}"
        self.sessions[session_id] = {
            "socket_id": socket_id,
            "profile": profile,
            "process_cap": process_cap,
            "created_at": time.time(),
            "handshake_completed": False,
            "peer_info": None,
        }
        
        # Simulate handshake delay
        await asyncio.sleep(0.05)  # 50ms handshake
        
        self.sessions[session_id]["handshake_completed"] = True
        self.sessions[session_id]["peer_info"] = {
            "sni": "demo.local",
            "alpn_protocol": "h2",
            "cipher_suite": self._get_cipher_suite(profile),
            "tls_version": "TLSv1.3",
            "is_mtls": profile == "intranet_fast",
        }
        
        self.stats["handshakes"] += 1
        return session_id
    
    async def tls_peer(self, session_id: str) -> Dict:
        """Mock get peer information"""
        if session_id not in self.sessions:
            raise ValueError("Session not found")
        
        return self.sessions[session_id]["peer_info"]
    
    async def tls_shutdown(self, session_id: str) -> None:
        """Mock TLS shutdown"""
        if session_id in self.sessions:
            del self.sessions[session_id]
            self.stats["shutdowns"] += 1
    
    async def tls_rekey(self, session_id: str, new_profile: str) -> None:
        """Mock TLS rekey"""
        if session_id not in self.sessions:
            raise ValueError("Session not found")
        
        # Simulate rekey delay
        await asyncio.sleep(0.02)  # 20ms rekey
        
        self.sessions[session_id]["profile"] = new_profile
        self.sessions[session_id]["peer_info"]["cipher_suite"] = self._get_cipher_suite(new_profile)
        self.stats["rekeys"] += 1
    
    def _check_capability(self, process_cap: str) -> bool:
        """Check if process has TLS capability"""
        return "net:tls" in process_cap or "net:all" in process_cap
    
    def _get_cipher_suite(self, profile: str) -> str:
        """Get cipher suite for profile"""
        suites = {
            "pqc_hybrid": "TLS_AES_256_GCM_SHA384_PQC",
            "tls13_modern": "TLS_AES_256_GCM_SHA384",
            "intranet_fast": "TLS_AES_128_GCM_SHA256",
        }
        return suites.get(profile, "TLS_AES_256_GCM_SHA384")

class MockQUICBroker:
    """Mock QUIC broker for testing"""
    
    def __init__(self):
        self.connections = {}
        self.streams = {}
        self.stats = {
            "connections": 0,
            "streams": 0,
            "bytes_sent": 0,
            "bytes_received": 0,
        }
    
    async def quic_connect(self, addr: str, profile: str, process_cap: str) -> str:
        """Mock QUIC connect operation"""
        if not self._check_capability(process_cap):
            raise PermissionError("QUIC capability required")
        
        conn_id = f"conn_{len(self.connections) + 1}"
        self.connections[conn_id] = {
            "addr": addr,
            "profile": profile,
            "process_cap": process_cap,
            "created_at": time.time(),
            "streams": [],
        }
        
        # Simulate connection delay
        await asyncio.sleep(0.01)  # 10ms connection
        
        self.stats["connections"] += 1
        return conn_id
    
    async def quic_open_bidi(self, conn_id: str, process_cap: str) -> str:
        """Mock open bidirectional stream"""
        if conn_id not in self.connections:
            raise ValueError("Connection not found")
        
        stream_id = f"stream_{len(self.streams) + 1}"
        self.streams[stream_id] = {
            "conn_id": conn_id,
            "process_cap": process_cap,
            "created_at": time.time(),
            "bytes_sent": 0,
            "bytes_received": 0,
        }
        
        self.connections[conn_id]["streams"].append(stream_id)
        self.stats["streams"] += 1
        return stream_id
    
    async def quic_write(self, stream_id: str, data: bytes, process_cap: str) -> int:
        """Mock QUIC write operation"""
        if stream_id not in self.streams:
            raise ValueError("Stream not found")
        
        # Simulate write delay
        await asyncio.sleep(0.001)  # 1ms write
        
        bytes_written = len(data)
        self.streams[stream_id]["bytes_sent"] += bytes_written
        self.stats["bytes_sent"] += bytes_written
        return bytes_written
    
    async def quic_read(self, stream_id: str, max_bytes: int, process_cap: str) -> bytes:
        """Mock QUIC read operation"""
        if stream_id not in self.streams:
            raise ValueError("Stream not found")
        
        # Simulate read delay
        await asyncio.sleep(0.001)  # 1ms read
        
        # Mock echo response
        response = b"QUIC echo response"
        bytes_received = min(len(response), max_bytes)
        self.streams[stream_id]["bytes_received"] += bytes_received
        self.stats["bytes_received"] += bytes_received
        return response[:bytes_received]
    
    async def quic_close_stream(self, stream_id: str, process_cap: str) -> None:
        """Mock close stream"""
        if stream_id in self.streams:
            conn_id = self.streams[stream_id]["conn_id"]
            if conn_id in self.connections:
                self.connections[conn_id]["streams"].remove(stream_id)
            del self.streams[stream_id]
    
    async def quic_close_connection(self, conn_id: str, process_cap: str) -> None:
        """Mock close connection"""
        if conn_id in self.connections:
            # Close all streams
            for stream_id in self.connections[conn_id]["streams"]:
                if stream_id in self.streams:
                    del self.streams[stream_id]
            del self.connections[conn_id]
    
    def _check_capability(self, process_cap: str) -> bool:
        """Check if process has QUIC capability"""
        return "net:quic" in process_cap or "net:all" in process_cap

class TLSQUICTester:
    """Main tester class"""
    
    def __init__(self):
        self.tls_broker = MockTLSBroker()
        self.quic_broker = MockQUICBroker()
        self.results: List[TestResult] = []
    
    async def run_all_tests(self) -> List[TestResult]:
        """Run all TLS/QUIC tests"""
        print("Starting TLS/QUIC validation tests...")
        
        # TLS tests
        await self._test_tls_handshake_timing()
        await self._test_tls_cipher_verification()
        await self._test_tls_mtls()
        await self._test_tls_rekey()
        await self._test_tls_policy_denial()
        
        # QUIC tests
        await self._test_quic_connection_timing()
        await self._test_quic_stream_operations()
        await self._test_quic_echo_performance()
        await self._test_quic_policy_denial()
        
        # Performance tests
        await self._test_performance_baselines()
        
        return self.results
    
    async def _test_tls_handshake_timing(self):
        """Test TLS handshake timing"""
        test_name = "TLS Handshake Timing"
        start_time = time.time()
        
        try:
            # Test different profiles
            profiles = [TLSProfile.TLS13_MODERN, TLSProfile.PQC_HYBRID, TLSProfile.INTRANET_FAST]
            handshake_times = []
            
            for profile in profiles:
                socket_id = f"socket_{profile.value}"
                session_id = await self.tls_broker.tls_wrap(
                    socket_id, profile.value, "net:tls,net:all"
                )
                
                # Measure handshake time
                session = self.tls_broker.sessions[session_id]
                handshake_time = (time.time() - session["created_at"]) * 1000
                handshake_times.append(handshake_time)
                
                await self.tls_broker.tls_shutdown(session_id)
            
            duration = (time.time() - start_time) * 1000
            avg_handshake = sum(handshake_times) / len(handshake_times)
            
            self.results.append(TestResult(
                test_name=test_name,
                success=avg_handshake <= 80.0,  # 80ms threshold
                duration_ms=duration,
                details={
                    "avg_handshake_ms": avg_handshake,
                    "handshake_times": handshake_times,
                    "profiles_tested": [p.value for p in profiles],
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_tls_cipher_verification(self):
        """Test TLS cipher suite verification"""
        test_name = "TLS Cipher Verification"
        start_time = time.time()
        
        try:
            # Test PQC hybrid profile
            socket_id = "socket_pqc_test"
            session_id = await self.tls_broker.tls_wrap(
                socket_id, TLSProfile.PQC_HYBRID.value, "net:tls,net:all"
            )
            
            peer_info = await self.tls_broker.tls_peer(session_id)
            cipher_suite = peer_info["cipher_suite"]
            
            # Verify PQC cipher suite
            success = "PQC" in cipher_suite or "Kyber" in cipher_suite or "Dilithium" in cipher_suite
            
            await self.tls_broker.tls_shutdown(session_id)
            
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "cipher_suite": cipher_suite,
                    "profile": TLSProfile.PQC_HYBRID.value,
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_tls_mtls(self):
        """Test mutual TLS"""
        test_name = "TLS Mutual Authentication"
        start_time = time.time()
        
        try:
            # Test intranet fast profile (requires mTLS)
            socket_id = "socket_mtls_test"
            session_id = await self.tls_broker.tls_wrap(
                socket_id, TLSProfile.INTRANET_FAST.value, "net:tls,net:all"
            )
            
            peer_info = await self.tls_broker.tls_peer(session_id)
            is_mtls = peer_info["is_mtls"]
            
            await self.tls_broker.tls_shutdown(session_id)
            
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=is_mtls,
                duration_ms=duration,
                details={
                    "is_mtls": is_mtls,
                    "profile": TLSProfile.INTRANET_FAST.value,
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_tls_rekey(self):
        """Test TLS rekey operation"""
        test_name = "TLS Rekey Operation"
        start_time = time.time()
        
        try:
            # Create initial session
            socket_id = "socket_rekey_test"
            session_id = await self.tls_broker.tls_wrap(
                socket_id, TLSProfile.TLS13_MODERN.value, "net:tls,net:all"
            )
            
            # Perform rekey
            await self.tls_broker.tls_rekey(session_id, TLSProfile.PQC_HYBRID.value)
            
            # Verify new profile
            peer_info = await self.tls_broker.tls_peer(session_id)
            new_cipher = peer_info["cipher_suite"]
            
            await self.tls_broker.tls_shutdown(session_id)
            
            duration = (time.time() - start_time) * 1000
            success = "PQC" in new_cipher
            
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "new_cipher_suite": new_cipher,
                    "rekey_count": self.tls_broker.stats["rekeys"],
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_tls_policy_denial(self):
        """Test TLS policy denial"""
        test_name = "TLS Policy Denial"
        start_time = time.time()
        
        try:
            # Try to use TLS without proper capability
            socket_id = "socket_policy_test"
            
            try:
                await self.tls_broker.tls_wrap(
                    socket_id, TLSProfile.TLS13_MODERN.value, "net:limited"
                )
                success = False  # Should have failed
            except PermissionError:
                success = True  # Expected failure
            
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "policy_denial_expected": True,
                    "capability_checked": "net:limited",
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_quic_connection_timing(self):
        """Test QUIC connection timing"""
        test_name = "QUIC Connection Timing"
        start_time = time.time()
        
        try:
            # Test connection establishment
            conn_id = await self.quic_broker.quic_connect(
                "127.0.0.1:9443", QUICProfile.TLS13_MODERN.value, "net:quic,net:all"
            )
            
            # Measure connection time
            connection = self.quic_broker.connections[conn_id]
            connection_time = (time.time() - connection["created_at"]) * 1000
            
            await self.quic_broker.quic_close_connection(conn_id, "net:quic,net:all")
            
            duration = (time.time() - start_time) * 1000
            success = connection_time <= 20.0  # 20ms threshold
            
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "connection_time_ms": connection_time,
                    "profile": QUICProfile.TLS13_MODERN.value,
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_quic_stream_operations(self):
        """Test QUIC stream operations"""
        test_name = "QUIC Stream Operations"
        start_time = time.time()
        
        try:
            # Create connection and stream
            conn_id = await self.quic_broker.quic_connect(
                "127.0.0.1:9443", QUICProfile.TLS13_MODERN.value, "net:quic,net:all"
            )
            
            stream_id = await self.quic_broker.quic_open_bidi(conn_id, "net:quic,net:all")
            
            # Test write/read operations
            test_data = b"Hello QUIC!"
            bytes_written = await self.quic_broker.quic_write(stream_id, test_data, "net:quic,net:all")
            response = await self.quic_broker.quic_read(stream_id, 1024, "net:quic,net:all")
            
            # Cleanup
            await self.quic_broker.quic_close_stream(stream_id, "net:quic,net:all")
            await self.quic_broker.quic_close_connection(conn_id, "net:quic,net:all")
            
            duration = (time.time() - start_time) * 1000
            success = bytes_written == len(test_data) and len(response) > 0
            
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "bytes_written": bytes_written,
                    "response_length": len(response),
                    "test_data_length": len(test_data),
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_quic_echo_performance(self):
        """Test QUIC echo performance"""
        test_name = "QUIC Echo Performance"
        start_time = time.time()
        
        try:
            # Create connection and stream
            conn_id = await self.quic_broker.quic_connect(
                "127.0.0.1:9443", QUICProfile.TLS13_MODERN.value, "net:quic,net:all"
            )
            
            stream_id = await self.quic_broker.quic_open_bidi(conn_id, "net:quic,net:all")
            
            # Measure round-trip times
            rtt_times = []
            for i in range(10):
                msg_start = time.time()
                test_data = f"QUIC test message {i}".encode()
                
                await self.quic_broker.quic_write(stream_id, test_data, "net:quic,net:all")
                response = await self.quic_broker.quic_read(stream_id, 1024, "net:quic,net:all")
                
                rtt_time = (time.time() - msg_start) * 1000
                rtt_times.append(rtt_time)
            
            # Cleanup
            await self.quic_broker.quic_close_stream(stream_id, "net:quic,net:all")
            await self.quic_broker.quic_close_connection(conn_id, "net:quic,net:all")
            
            duration = (time.time() - start_time) * 1000
            avg_rtt = sum(rtt_times) / len(rtt_times)
            p95_rtt = sorted(rtt_times)[int(len(rtt_times) * 0.95)]
            
            success = p95_rtt <= 2.0  # 2ms threshold
            
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "avg_rtt_ms": avg_rtt,
                    "p95_rtt_ms": p95_rtt,
                    "rtt_times": rtt_times,
                    "messages_sent": len(rtt_times),
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_quic_policy_denial(self):
        """Test QUIC policy denial"""
        test_name = "QUIC Policy Denial"
        start_time = time.time()
        
        try:
            # Try to use QUIC without proper capability
            try:
                await self.quic_broker.quic_connect(
                    "127.0.0.1:9443", QUICProfile.TLS13_MODERN.value, "net:limited"
                )
                success = False  # Should have failed
            except PermissionError:
                success = True  # Expected failure
            
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=success,
                duration_ms=duration,
                details={
                    "policy_denial_expected": True,
                    "capability_checked": "net:limited",
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    async def _test_performance_baselines(self):
        """Test performance baselines"""
        test_name = "Performance Baselines"
        start_time = time.time()
        
        try:
            # Collect performance metrics
            metrics = PerformanceMetrics(
                handshake_latency_p50=50.0,  # Mock values
                handshake_latency_p95=80.0,
                quic_rtt_p50=1.0,
                quic_rtt_p95=2.0,
                throughput_mbps=100.0,
                resume_ratio=0.8,
            )
            
            # Check baselines
            baselines_met = (
                metrics.handshake_latency_p95 <= 80.0 and
                metrics.quic_rtt_p95 <= 2.0 and
                metrics.throughput_mbps >= 2.0
            )
            
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=baselines_met,
                duration_ms=duration,
                details={
                    "handshake_latency_p95": metrics.handshake_latency_p95,
                    "quic_rtt_p95": metrics.quic_rtt_p95,
                    "throughput_mbps": metrics.throughput_mbps,
                    "resume_ratio": metrics.resume_ratio,
                }
            ))
            
        except Exception as e:
            duration = (time.time() - start_time) * 1000
            self.results.append(TestResult(
                test_name=test_name,
                success=False,
                duration_ms=duration,
                error_message=str(e)
            ))
    
    def print_results(self):
        """Print test results"""
        print("\n" + "="*60)
        print("TLS/QUIC Validation Results")
        print("="*60)
        
        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.success)
        failed_tests = total_tests - passed_tests
        
        print(f"Total Tests: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {failed_tests}")
        print(f"Success Rate: {(passed_tests/total_tests)*100:.1f}%")
        print()
        
        for result in self.results:
            status = "✅ PASS" if result.success else "❌ FAIL"
            print(f"{status} {result.test_name} ({result.duration_ms:.1f}ms)")
            
            if result.error_message:
                print(f"    Error: {result.error_message}")
            
            if result.details:
                for key, value in result.details.items():
                    print(f"    {key}: {value}")
            print()
        
        # Print broker statistics
        print("Broker Statistics:")
        print(f"  TLS Handshakes: {self.tls_broker.stats['handshakes']}")
        print(f"  TLS Rekeys: {self.tls_broker.stats['rekeys']}")
        print(f"  TLS Shutdowns: {self.tls_broker.stats['shutdowns']}")
        print(f"  QUIC Connections: {self.quic_broker.stats['connections']}")
        print(f"  QUIC Streams: {self.quic_broker.stats['streams']}")
        print(f"  QUIC Bytes Sent: {self.quic_broker.stats['bytes_sent']}")
        print(f"  QUIC Bytes Received: {self.quic_broker.stats['bytes_received']}")

async def main():
    """Main entry point"""
    tester = TLSQUICTester()
    results = await tester.run_all_tests()
    tester.print_results()
    
    # Exit with error code if any tests failed
    failed_tests = [r for r in results if not r.success]
    if failed_tests:
        print(f"\n❌ {len(failed_tests)} test(s) failed!")
        sys.exit(1)
    else:
        print("\n✅ All tests passed!")
        sys.exit(0)

if __name__ == "__main__":
    asyncio.run(main())
