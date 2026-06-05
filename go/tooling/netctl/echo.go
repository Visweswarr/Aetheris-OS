//! Echo server and client implementations for netctl
//! 
//! This module provides TCP and UDP echo server and client functionality
//! for testing the Aetheris OS networking subsystem.

package main

import (
	"context"
	"fmt"
	"net"
	"time"
)

// Start TCP echo server
func startTCPEchoServer(addr string, verbose bool) error {
	// Parse address
	listener, err := net.Listen("tcp", addr)
	if err != nil {
		return fmt.Errorf("failed to listen on %s: %v", addr, err)
	}
	defer listener.Close()

	if verbose {
		fmt.Printf("TCP echo server listening on %s\n", addr)
	}

	// Accept connections
	for {
		conn, err := listener.Accept()
		if err != nil {
			if verbose {
				fmt.Printf("Failed to accept connection: %v\n", err)
			}
			continue
		}

		// Handle connection in goroutine
		go handleTCPConnection(conn, verbose)
	}
}

// Handle TCP connection
func handleTCPConnection(conn net.Conn, verbose bool) {
	defer conn.Close()

	clientAddr := conn.RemoteAddr().String()
	if verbose {
		fmt.Printf("New TCP connection from %s\n", clientAddr)
	}

	// Set read timeout
	conn.SetReadDeadline(time.Now().Add(30 * time.Second))

	// Echo loop
	buffer := make([]byte, 4096)
	for {
		n, err := conn.Read(buffer)
		if err != nil {
			if verbose {
				fmt.Printf("TCP connection from %s closed: %v\n", clientAddr, err)
			}
			break
		}

		// Echo data back
		_, err = conn.Write(buffer[:n])
		if err != nil {
			if verbose {
				fmt.Printf("Failed to write to TCP connection from %s: %v\n", clientAddr, err)
			}
			break
		}

		if verbose {
			fmt.Printf("TCP echoed %d bytes to %s\n", n, clientAddr)
		}
	}
}

// Start UDP echo server
func startUDPEchoServer(addr string, verbose bool) error {
	// Parse address
	udpAddr, err := net.ResolveUDPAddr("udp", addr)
	if err != nil {
		return fmt.Errorf("failed to resolve UDP address %s: %v", addr, err)
	}

	conn, err := net.ListenUDP("udp", udpAddr)
	if err != nil {
		return fmt.Errorf("failed to listen on UDP %s: %v", addr, err)
	}
	defer conn.Close()

	if verbose {
		fmt.Printf("UDP echo server listening on %s\n", addr)
	}

	// Echo loop
	buffer := make([]byte, 4096)
	for {
		n, clientAddr, err := conn.ReadFromUDP(buffer)
		if err != nil {
			if verbose {
				fmt.Printf("Failed to read from UDP: %v\n", err)
			}
			continue
		}

		// Echo data back
		_, err = conn.WriteToUDP(buffer[:n], clientAddr)
		if err != nil {
			if verbose {
				fmt.Printf("Failed to write to UDP client %s: %v\n", clientAddr, err)
			}
			continue
		}

		if verbose {
			fmt.Printf("UDP echoed %d bytes to %s\n", n, clientAddr)
		}
	}
}

// Start TCP echo client
func startTCPEchoClient(addr, message string, count int, interval time.Duration, verbose bool) error {
	// Connect to server
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return fmt.Errorf("failed to connect to %s: %v", addr, err)
	}
	defer conn.Close()

	if verbose {
		fmt.Printf("Connected to TCP echo server at %s\n", addr)
	}

	// Send messages
	for i := 0; i < count; i++ {
		start := time.Now()
		
		// Send message
		_, err := conn.Write([]byte(message))
		if err != nil {
			return fmt.Errorf("failed to send message: %v", err)
		}

		// Read response
		buffer := make([]byte, len(message))
		n, err := conn.Read(buffer)
		if err != nil {
			return fmt.Errorf("failed to read response: %v", err)
		}

		duration := time.Since(start)
		
		if verbose {
			fmt.Printf("Message %d: sent '%s', received '%s' in %v\n", 
				i+1, message, string(buffer[:n]), duration)
		} else {
			fmt.Printf("Message %d: %v\n", i+1, duration)
		}

		// Wait for interval
		if interval > 0 && i < count-1 {
			time.Sleep(interval)
		}
	}

	return nil
}

// Start UDP echo client
func startUDPEchoClient(addr, message string, count int, interval time.Duration, verbose bool) error {
	// Connect to server
	conn, err := net.Dial("udp", addr)
	if err != nil {
		return fmt.Errorf("failed to connect to %s: %v", addr, err)
	}
	defer conn.Close()

	if verbose {
		fmt.Printf("Connected to UDP echo server at %s\n", addr)
	}

	// Send messages
	for i := 0; i < count; i++ {
		start := time.Now()
		
		// Send message
		_, err := conn.Write([]byte(message))
		if err != nil {
			return fmt.Errorf("failed to send message: %v", err)
		}

		// Read response
		buffer := make([]byte, len(message))
		conn.SetReadDeadline(time.Now().Add(5 * time.Second))
		n, err := conn.Read(buffer)
		if err != nil {
			return fmt.Errorf("failed to read response: %v", err)
		}

		duration := time.Since(start)
		
		if verbose {
			fmt.Printf("Message %d: sent '%s', received '%s' in %v\n", 
				i+1, message, string(buffer[:n]), duration)
		} else {
			fmt.Printf("Message %d: %v\n", i+1, duration)
		}

		// Wait for interval
		if interval > 0 && i < count-1 {
			time.Sleep(interval)
		}
	}

	return nil
}

// Benchmark TCP echo performance
func benchmarkTCPEcho(addr string, messageSize int, count int) error {
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return fmt.Errorf("failed to connect to %s: %v", addr, err)
	}
	defer conn.Close()

	message := make([]byte, messageSize)
	for i := range message {
		message[i] = byte(i % 256)
	}

	var totalDuration time.Duration
	var minDuration, maxDuration time.Duration

	for i := 0; i < count; i++ {
		start := time.Now()
		
		// Send message
		_, err := conn.Write(message)
		if err != nil {
			return fmt.Errorf("failed to send message: %v", err)
		}

		// Read response
		buffer := make([]byte, messageSize)
		_, err = conn.Read(buffer)
		if err != nil {
			return fmt.Errorf("failed to read response: %v", err)
		}

		duration := time.Since(start)
		totalDuration += duration

		if i == 0 {
			minDuration = duration
			maxDuration = duration
		} else {
			if duration < minDuration {
				minDuration = duration
			}
			if duration > maxDuration {
				maxDuration = duration
			}
		}
	}

	avgDuration := totalDuration / time.Duration(count)
	
	fmt.Printf("TCP Echo Benchmark Results:\n")
	fmt.Printf("  Messages: %d\n", count)
	fmt.Printf("  Message Size: %d bytes\n", messageSize)
	fmt.Printf("  Total Time: %v\n", totalDuration)
	fmt.Printf("  Average RTT: %v\n", avgDuration)
	fmt.Printf("  Min RTT: %v\n", minDuration)
	fmt.Printf("  Max RTT: %v\n", maxDuration)
	fmt.Printf("  Throughput: %.2f MB/s\n", 
		float64(count*messageSize*2)/(totalDuration.Seconds()*1024*1024))

	return nil
}

// Benchmark UDP echo performance
func benchmarkUDPEcho(addr string, messageSize int, count int) error {
	conn, err := net.Dial("udp", addr)
	if err != nil {
		return fmt.Errorf("failed to connect to %s: %v", addr, err)
	}
	defer conn.Close()

	message := make([]byte, messageSize)
	for i := range message {
		message[i] = byte(i % 256)
	}

	var totalDuration time.Duration
	var minDuration, maxDuration time.Duration

	for i := 0; i < count; i++ {
		start := time.Now()
		
		// Send message
		_, err := conn.Write(message)
		if err != nil {
			return fmt.Errorf("failed to send message: %v", err)
		}

		// Read response
		buffer := make([]byte, messageSize)
		conn.SetReadDeadline(time.Now().Add(5 * time.Second))
		_, err = conn.Read(buffer)
		if err != nil {
			return fmt.Errorf("failed to read response: %v", err)
		}

		duration := time.Since(start)
		totalDuration += duration

		if i == 0 {
			minDuration = duration
			maxDuration = duration
		} else {
			if duration < minDuration {
				minDuration = duration
			}
			if duration > maxDuration {
				maxDuration = duration
			}
		}
	}

	avgDuration := totalDuration / time.Duration(count)
	
	fmt.Printf("UDP Echo Benchmark Results:\n")
	fmt.Printf("  Messages: %d\n", count)
	fmt.Printf("  Message Size: %d bytes\n", messageSize)
	fmt.Printf("  Total Time: %v\n", totalDuration)
	fmt.Printf("  Average RTT: %v\n", avgDuration)
	fmt.Printf("  Min RTT: %v\n", minDuration)
	fmt.Printf("  Max RTT: %v\n", maxDuration)
	fmt.Printf("  Throughput: %.2f MB/s\n", 
		float64(count*messageSize*2)/(totalDuration.Seconds()*1024*1024))

	return nil
}
