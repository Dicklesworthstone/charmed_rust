// Log capture program - captures logging library behaviors
package main

import (
	"bytes"
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/charmbracelet/log"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("log", "0.4.0")

	// Capture log level tests
	captureLogLevelTests(fixtures)

	// Capture log format tests
	captureLogFormatTests(fixtures)

	// Capture structured logging tests
	captureStructuredLoggingTests(fixtures)

	// Capture timestamp tests
	captureTimestampTests(fixtures)

	// Capture prefix tests
	capturePrefixTests(fixtures)

	// Capture caller tests
	captureCallerTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureLogLevelTests(fs *capture.FixtureSet) {
	// Test log level constants
	levels := []struct {
		name  string
		level log.Level
	}{
		{"Debug", log.DebugLevel},
		{"Info", log.InfoLevel},
		{"Warn", log.WarnLevel},
		{"Error", log.ErrorLevel},
		{"Fatal", log.FatalLevel},
	}

	for _, l := range levels {
		fs.AddTestWithCategory(fmt.Sprintf("level_%s", strings.ToLower(l.name)), "unit",
			map[string]interface{}{
				"level_name": l.name,
			},
			map[string]interface{}{
				"level_value":  int(l.level),
				"level_string": l.level.String(),
			},
		)
	}

	// Test level parsing
	levelStrings := []string{"debug", "info", "warn", "error", "fatal", "DEBUG", "INFO"}
	for _, ls := range levelStrings {
		parsed, err := log.ParseLevel(ls)
		errStr := ""
		if err != nil {
			errStr = err.Error()
		}
		fs.AddTestWithCategory(fmt.Sprintf("parse_level_%s", strings.ToLower(ls)), "unit",
			map[string]interface{}{
				"input": ls,
			},
			map[string]interface{}{
				"parsed_value": int(parsed),
				"error":        errStr,
			},
		)
	}
}

func captureLogFormatTests(fs *capture.FixtureSet) {
	// Test 1: Text format output
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.SetReportCaller(false)
		logger.SetLevel(log.DebugLevel)
		logger.Info("test message")
		output := buf.String()
		fs.AddTestWithCategory("format_text_basic", "unit",
			map[string]interface{}{
				"message": "test message",
				"level":   "info",
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_level": strings.Contains(output, "INFO"),
				"contains_msg":   strings.Contains(output, "test message"),
			},
		)
	}

	// Test 2: Debug level message
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.SetLevel(log.DebugLevel)
		logger.Debug("debug message")
		output := buf.String()
		fs.AddTestWithCategory("format_debug", "unit",
			map[string]interface{}{
				"message": "debug message",
				"level":   "debug",
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_level": strings.Contains(output, "DEBU"),
			},
		)
	}

	// Test 3: Warn level message
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Warn("warning message")
		output := buf.String()
		fs.AddTestWithCategory("format_warn", "unit",
			map[string]interface{}{
				"message": "warning message",
				"level":   "warn",
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_level": strings.Contains(output, "WARN"),
			},
		)
	}

	// Test 4: Error level message
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Error("error message")
		output := buf.String()
		fs.AddTestWithCategory("format_error", "unit",
			map[string]interface{}{
				"message": "error message",
				"level":   "error",
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_level": strings.Contains(output, "ERRO"),
			},
		)
	}

	// Test 5: Message with format arguments
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Infof("count is %d", 42)
		output := buf.String()
		fs.AddTestWithCategory("format_printf", "unit",
			map[string]interface{}{
				"format": "count is %d",
				"args":   []interface{}{42},
			},
			map[string]interface{}{
				"output":       strings.TrimSpace(output),
				"contains_42":  strings.Contains(output, "42"),
			},
		)
	}
}

func captureStructuredLoggingTests(fs *capture.FixtureSet) {
	// Test 1: Single field
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Info("message", "key", "value")
		output := buf.String()
		fs.AddTestWithCategory("structured_single_field", "unit",
			map[string]interface{}{
				"message": "message",
				"fields":  map[string]interface{}{"key": "value"},
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_key":   strings.Contains(output, "key"),
				"contains_value": strings.Contains(output, "value"),
			},
		)
	}

	// Test 2: Multiple fields
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Info("event", "user", "alice", "action", "login", "status", "success")
		output := buf.String()
		fs.AddTestWithCategory("structured_multiple_fields", "unit",
			map[string]interface{}{
				"message": "event",
				"fields": map[string]interface{}{
					"user":   "alice",
					"action": "login",
					"status": "success",
				},
			},
			map[string]interface{}{
				"output":          strings.TrimSpace(output),
				"contains_user":   strings.Contains(output, "user"),
				"contains_alice":  strings.Contains(output, "alice"),
				"contains_action": strings.Contains(output, "action"),
			},
		)
	}

	// Test 3: Numeric field
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Info("metrics", "count", 100, "rate", 3.14)
		output := buf.String()
		fs.AddTestWithCategory("structured_numeric_fields", "unit",
			map[string]interface{}{
				"message": "metrics",
				"fields": map[string]interface{}{
					"count": 100,
					"rate":  3.14,
				},
			},
			map[string]interface{}{
				"output":        strings.TrimSpace(output),
				"contains_100":  strings.Contains(output, "100"),
				"contains_3.14": strings.Contains(output, "3.14"),
			},
		)
	}

	// Test 4: Boolean field
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Info("config", "enabled", true, "debug", false)
		output := buf.String()
		fs.AddTestWithCategory("structured_boolean_fields", "unit",
			map[string]interface{}{
				"message": "config",
				"fields": map[string]interface{}{
					"enabled": true,
					"debug":   false,
				},
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_true":  strings.Contains(output, "true"),
				"contains_false": strings.Contains(output, "false"),
			},
		)
	}

	// Test 5: With pre-set fields
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger = logger.With("component", "auth")
		logger.Info("request")
		output := buf.String()
		fs.AddTestWithCategory("structured_with_fields", "unit",
			map[string]interface{}{
				"message":     "request",
				"with_fields": map[string]interface{}{"component": "auth"},
			},
			map[string]interface{}{
				"output":             strings.TrimSpace(output),
				"contains_component": strings.Contains(output, "component"),
				"contains_auth":      strings.Contains(output, "auth"),
			},
		)
	}
}

func captureTimestampTests(fs *capture.FixtureSet) {
	// Test 1: Timestamp enabled
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(true)
		logger.SetTimeFormat(time.Kitchen)
		logger.Info("with timestamp")
		output := buf.String()
		fs.AddTestWithCategory("timestamp_enabled", "unit",
			map[string]interface{}{
				"timestamp_enabled": true,
				"format":            "Kitchen",
			},
			map[string]interface{}{
				"output":             strings.TrimSpace(output),
				"has_time_separator": strings.Contains(output, ":"),
			},
		)
	}

	// Test 2: Timestamp disabled
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Info("no timestamp")
		output := buf.String()
		fs.AddTestWithCategory("timestamp_disabled", "unit",
			map[string]interface{}{
				"timestamp_enabled": false,
			},
			map[string]interface{}{
				"output": strings.TrimSpace(output),
			},
		)
	}

	// Test 3: Custom time format
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(true)
		logger.SetTimeFormat("2006-01-02")
		logger.Info("custom format")
		output := buf.String()
		fs.AddTestWithCategory("timestamp_custom_format", "unit",
			map[string]interface{}{
				"timestamp_enabled": true,
				"format":            "2006-01-02",
			},
			map[string]interface{}{
				"output":       strings.TrimSpace(output),
				"contains_dash": strings.Contains(output, "-"),
			},
		)
	}
}

func capturePrefixTests(fs *capture.FixtureSet) {
	// Test 1: With prefix
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.SetPrefix("myapp")
		logger.Info("message")
		output := buf.String()
		fs.AddTestWithCategory("prefix_set", "unit",
			map[string]interface{}{
				"prefix":  "myapp",
				"message": "message",
			},
			map[string]interface{}{
				"output":          strings.TrimSpace(output),
				"contains_prefix": strings.Contains(output, "myapp"),
			},
		)
	}

	// Test 2: Without prefix
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.Info("no prefix")
		output := buf.String()
		fs.AddTestWithCategory("prefix_none", "unit",
			map[string]interface{}{
				"prefix":  "",
				"message": "no prefix",
			},
			map[string]interface{}{
				"output": strings.TrimSpace(output),
			},
		)
	}

	// Test 3: Emoji prefix
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.SetPrefix("🚀")
		logger.Info("rocket launch")
		output := buf.String()
		fs.AddTestWithCategory("prefix_emoji", "unit",
			map[string]interface{}{
				"prefix":  "🚀",
				"message": "rocket launch",
			},
			map[string]interface{}{
				"output": strings.TrimSpace(output),
			},
		)
	}
}

func captureCallerTests(fs *capture.FixtureSet) {
	// Test 1: Caller enabled
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.SetReportCaller(true)
		logger.Info("with caller")
		output := buf.String()
		fs.AddTestWithCategory("caller_enabled", "unit",
			map[string]interface{}{
				"caller_enabled": true,
			},
			map[string]interface{}{
				"output":         strings.TrimSpace(output),
				"contains_colon": strings.Contains(output, ":"),
				"contains_go":    strings.Contains(output, ".go"),
			},
		)
	}

	// Test 2: Caller disabled
	{
		var buf bytes.Buffer
		logger := log.New(&buf)
		logger.SetReportTimestamp(false)
		logger.SetReportCaller(false)
		logger.Info("no caller")
		output := buf.String()
		fs.AddTestWithCategory("caller_disabled", "unit",
			map[string]interface{}{
				"caller_enabled": false,
			},
			map[string]interface{}{
				"output": strings.TrimSpace(output),
			},
		)
	}
}
