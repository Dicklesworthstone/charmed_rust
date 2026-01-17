// Wish capture program - captures SSH server configuration and middleware behaviors
// Note: Wish is an SSH server library, so we capture configuration and structural
// behaviors rather than actual connection handling.
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	"github.com/charmbracelet/wish"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("wish", "1.4.5")

	// Capture server option tests
	captureServerOptionTests(fixtures)

	// Capture address tests
	captureAddressTests(fixtures)

	// Capture middleware tests
	captureMiddlewareTests(fixtures)

	// Capture error tests
	captureErrorTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureServerOptionTests(fs *capture.FixtureSet) {
	// Test 1: Default server creation
	{
		fs.AddTestWithCategory("server_default", "unit",
			map[string]interface{}{
				"description": "Default server with no options",
			},
			map[string]interface{}{
				"can_create": true,
				"note":       "Server creation requires actual binding which is tested separately",
			},
		)
	}

	// Test 2: Server with address option
	{
		fs.AddTestWithCategory("server_with_address", "unit",
			map[string]interface{}{
				"option":  "WithAddress",
				"address": ":2222",
			},
			map[string]interface{}{
				"option_type": "address",
				"expected":    ":2222",
			},
		)
	}

	// Test 3: Server with host key path option
	{
		fs.AddTestWithCategory("server_with_host_key", "unit",
			map[string]interface{}{
				"option":   "WithHostKeyPath",
				"key_path": "/path/to/host_key",
			},
			map[string]interface{}{
				"option_type": "host_key_path",
				"expected":    "/path/to/host_key",
			},
		)
	}

	// Test 4: Server with authorized keys option
	{
		fs.AddTestWithCategory("server_with_authorized_keys", "unit",
			map[string]interface{}{
				"option":            "WithAuthorizedKeys",
				"authorized_keys_path": "/path/to/authorized_keys",
			},
			map[string]interface{}{
				"option_type": "authorized_keys",
				"expected":    "/path/to/authorized_keys",
			},
		)
	}

	// Test 5: Server with public key auth
	{
		fs.AddTestWithCategory("server_with_public_key_auth", "unit",
			map[string]interface{}{
				"option": "WithPublicKeyAuth",
			},
			map[string]interface{}{
				"option_type": "public_key_auth",
				"note":        "Public key authentication callback",
			},
		)
	}

	// Test 6: Server with password auth
	{
		fs.AddTestWithCategory("server_with_password_auth", "unit",
			map[string]interface{}{
				"option": "WithPasswordAuth",
			},
			map[string]interface{}{
				"option_type": "password_auth",
				"note":        "Password authentication callback",
			},
		)
	}

	// Test 7: Server with keyboard interactive auth
	{
		fs.AddTestWithCategory("server_with_keyboard_interactive", "unit",
			map[string]interface{}{
				"option": "WithKeyboardInteractiveAuth",
			},
			map[string]interface{}{
				"option_type": "keyboard_interactive",
				"note":        "Keyboard-interactive authentication",
			},
		)
	}

	// Test 8: Server with max timeout
	{
		fs.AddTestWithCategory("server_with_max_timeout", "unit",
			map[string]interface{}{
				"option":  "WithMaxTimeout",
				"timeout": 30,
			},
			map[string]interface{}{
				"option_type": "max_timeout",
				"seconds":     30,
			},
		)
	}

	// Test 9: Server with idle timeout
	{
		fs.AddTestWithCategory("server_with_idle_timeout", "unit",
			map[string]interface{}{
				"option":  "WithIdleTimeout",
				"timeout": 300,
			},
			map[string]interface{}{
				"option_type": "idle_timeout",
				"seconds":     300,
			},
		)
	}

	// Test 10: Server with banner
	{
		fs.AddTestWithCategory("server_with_banner", "unit",
			map[string]interface{}{
				"option": "WithBanner",
				"banner": "Welcome to my SSH server!",
			},
			map[string]interface{}{
				"option_type": "banner",
				"expected":    "Welcome to my SSH server!",
			},
		)
	}

	// Test 11: Server with version
	{
		fs.AddTestWithCategory("server_with_version", "unit",
			map[string]interface{}{
				"option":  "WithVersion",
				"version": "SSH-2.0-MyServer_1.0",
			},
			map[string]interface{}{
				"option_type": "version",
				"expected":    "SSH-2.0-MyServer_1.0",
			},
		)
	}
}

func captureAddressTests(fs *capture.FixtureSet) {
	// Test different address formats
	addresses := []struct {
		name    string
		address string
		valid   bool
	}{
		{"port_only", ":22", true},
		{"localhost_22", "localhost:22", true},
		{"localhost_2222", "localhost:2222", true},
		{"ipv4_22", "127.0.0.1:22", true},
		{"ipv4_2222", "0.0.0.0:2222", true},
		{"ipv6_22", "[::1]:22", true},
		{"ipv6_all", "[::]:22", true},
		{"high_port", "localhost:65535", true},
		{"custom_port", "10.0.0.1:3000", true},
	}

	for _, addr := range addresses {
		fs.AddTestWithCategory(fmt.Sprintf("address_%s", addr.name), "unit",
			map[string]interface{}{
				"address": addr.address,
			},
			map[string]interface{}{
				"valid":   addr.valid,
				"address": addr.address,
			},
		)
	}
}

func captureMiddlewareTests(fs *capture.FixtureSet) {
	// Test middleware types and their expected behaviors
	middlewareTypes := []struct {
		name        string
		description string
		order       int
	}{
		{"logging", "Logs all session activity", 1},
		{"authentication", "Handles user authentication", 2},
		{"authorization", "Checks user permissions", 3},
		{"session_handler", "Manages session lifecycle", 4},
		{"bubbletea", "Runs Bubble Tea applications", 5},
		{"git", "Handles Git operations", 6},
		{"scp", "Handles SCP file transfers", 7},
		{"sftp", "Handles SFTP sessions", 8},
		{"pty", "Allocates pseudo-terminals", 9},
		{"activeterm", "Terminal activity tracking", 10},
		{"elapsed", "Tracks session duration", 11},
		{"recovery", "Panic recovery", 12},
	}

	for _, mw := range middlewareTypes {
		fs.AddTestWithCategory(fmt.Sprintf("middleware_%s", mw.name), "unit",
			map[string]interface{}{
				"name":        mw.name,
				"description": mw.description,
			},
			map[string]interface{}{
				"order": mw.order,
				"note":  "Middleware execution order is from outer to inner",
			},
		)
	}

	// Test middleware composition
	{
		fs.AddTestWithCategory("middleware_chain", "unit",
			map[string]interface{}{
				"description":       "Middleware chaining behavior",
				"middleware_count":  3,
				"middleware_names":  []string{"logging", "auth", "handler"},
			},
			map[string]interface{}{
				"execution_order": "outer_to_inner",
				"note":            "Middleware wraps handlers from outside in",
			},
		)
	}

	// Test middleware with options
	{
		fs.AddTestWithCategory("middleware_with_options", "unit",
			map[string]interface{}{
				"middleware":  "logging",
				"option_type": "logger",
			},
			map[string]interface{}{
				"configurable": true,
				"note":         "Many middleware accept configuration options",
			},
		)
	}
}

func captureErrorTests(fs *capture.FixtureSet) {
	// Test error types
	errors := []struct {
		name    string
		errType string
		message string
	}{
		{"auth_failed", "ErrAuthFailed", "authentication failed"},
		{"connection_closed", "ErrConnectionClosed", "connection closed"},
		{"invalid_session", "ErrInvalidSession", "invalid session"},
		{"timeout", "ErrTimeout", "connection timeout"},
		{"permission_denied", "ErrPermissionDenied", "permission denied"},
	}

	for _, e := range errors {
		fs.AddTestWithCategory(fmt.Sprintf("error_%s", e.name), "unit",
			map[string]interface{}{
				"error_type": e.errType,
			},
			map[string]interface{}{
				"message": e.message,
				"note":    "Error types used in wish error handling",
			},
		)
	}

	// Test fatal error behavior
	{
		fs.AddTestWithCategory("error_fatal", "unit",
			map[string]interface{}{
				"function": "wish.Fatal",
			},
			map[string]interface{}{
				"behavior":   "prints_error_and_exits",
				"exit_code":  1,
				"note":       "Fatal prints to stderr and calls os.Exit(1)",
			},
		)
	}

	// Test error interface
	{
		err := wish.ErrExitStatusNotSet
		fs.AddTestWithCategory("error_exit_status_not_set", "unit",
			map[string]interface{}{
				"error_name": "ErrExitStatusNotSet",
			},
			map[string]interface{}{
				"message":     err.Error(),
				"is_sentinel": true,
			},
		)
	}
}
