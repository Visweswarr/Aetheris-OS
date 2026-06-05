// Package intentclient provides a minimal helper to serialize an intent envelope
// and write it to INTENT_SOCKET. For now, supports to-file:// fallback for local testing.
package intentclient

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
	"runtime"

	cbor "github.com/fxamacker/cbor/v2"
)

type IntentEnvelopeV1 struct {
	V        uint8  `cbor:"v"`
	Captoken []byte `cbor:"captoken"`
	Payload  []byte `cbor:"payload"`
}

type IntentResponse struct {
	Status    string  `cbor:"status"`
	PreviewID *string `cbor:"preview_id,omitempty"`
	CommitID  *string `cbor:"commit_id,omitempty"`
	Message   *string `cbor:"message,omitempty"`
}

func writeFrame(w io.Writer, b []byte) error {
	var lenbuf [4]byte
	binary.BigEndian.PutUint32(lenbuf[:], uint32(len(b)))
	if _, err := w.Write(lenbuf[:]); err != nil { return err }
	_, err := w.Write(b)
	return err
}

func readFrame(r io.Reader) ([]byte, error) {
	var lenbuf [4]byte
	if _, err := io.ReadFull(r, lenbuf[:]); err != nil { return nil, err }
	rlen := binary.BigEndian.Uint32(lenbuf[:])
	buf := make([]byte, rlen)
	if _, err := io.ReadFull(r, buf); err != nil { return nil, err }
	return buf, nil
}

func SendIntent(env IntentEnvelopeV1, endpoint string) (*IntentResponse, error) {
	enc, err := cbor.Marshal(env)
	if err != nil { return nil, err }
	if len(endpoint) > 10 && endpoint[:10] == "to-file://" {
		path := endpoint[len("to-file://"):]
		f, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
		if err != nil { return nil, err }
		defer f.Close()
		if _, err := f.Write(append(enc, '\n')); err != nil { return nil, err }
		ok := "ok"
		return &IntentResponse{Status: "ok", Message: &ok}, nil
	}
	if len(endpoint) > 7 && endpoint[:7] == "unix://" && runtime.GOOS != "windows" {
		addr := endpoint[len("unix://"):]
		conn, err := net.Dial("unix", addr)
		if err != nil { return nil, err }
		defer conn.Close()
		if err := writeFrame(conn, enc); err != nil { return nil, err }
		respb, err := readFrame(conn)
		if err != nil { return nil, err }
		var resp IntentResponse
		if err := cbor.Unmarshal(respb, &resp); err != nil { return nil, err }
		return &resp, nil
	}
	if len(endpoint) > 7 && endpoint[:7] == "pipe://" && runtime.GOOS == "windows" {
		// Windows named pipe via go-winio
		conn, err := winioDialPipe(endpoint[len("pipe://"):])
		if err != nil { return nil, err }
		defer conn.Close()
		if err := writeFrame(conn, enc); err != nil { return nil, err }
		respb, err := readFrame(conn)
		if err != nil { return nil, err }
		var resp IntentResponse
		if err := cbor.Unmarshal(respb, &resp); err != nil { return nil, err }
		return &resp, nil
	}
	return nil, fmt.Errorf("unsupported endpoint: %s", endpoint)
}

//go:build windows
// +build windows

package intentclient

import (
	winio "github.com/Microsoft/go-winio"
)

func winioDialPipe(path string) (io.ReadWriteCloser, error) {
	return winio.DialPipe(path, nil)
}

//go:build !windows
// +build !windows

package intentclient

import "io"

func winioDialPipe(path string) (io.ReadWriteCloser, error) { return nil, errors.New("no pipe on non-windows") }
