package main

import (
	"fmt"
	"os"
	"path/filepath"

	"aetheris/bindings"
)

func main() {
	endpoint := os.Getenv("INTENT_SOCKET")
	if endpoint == "" { fmt.Println("missing INTENT_SOCKET"); os.Exit(2) }
	var tok []byte
	if p := os.Getenv("AICORE_CAPTOKEN"); p != "" {
		b, _ := os.ReadFile(p); tok = b
	}
	// Dummy payload
	payload := []byte{0xA0} // CBOR empty map
	env := intentclient.IntentEnvelopeV1{ V: 1, Captoken: tok, Payload: payload }
	resp, err := intentclient.SendIntent(env, endpoint)
	if err != nil { fmt.Println("err:", err); os.Exit(1) }
	fmt.Println(resp.Status)
}
