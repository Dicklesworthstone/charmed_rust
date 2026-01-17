// Charmed log capture program - captures logging level and formatting behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	"github.com/charmbracelet/log"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("charmed_log", "0.4.0")

	// Capture log level tests
	captureLogLevelTests(fixtures)

	// Capture level parsing tests
	captureLevelParsingTests(fixtures)

	// Capture level string tests
	captureLevelStringTests(fixtures)

	// Capture level comparison tests
	captureLevelComparisonTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureLogLevelTests(fs *capture.FixtureSet) {
	// Capture log level constants
	levels := []struct {
		name  string
		value log.Level
	}{
		{"DebugLevel", log.DebugLevel},
		{"InfoLevel", log.InfoLevel},
		{"WarnLevel", log.WarnLevel},
		{"ErrorLevel", log.ErrorLevel},
		{"FatalLevel", log.FatalLevel},
	}

	for _, lvl := range levels {
		fs.AddTestWithCategory(fmt.Sprintf("level_%s", lvl.name), "unit",
			map[string]string{
				"level": lvl.name,
			},
			map[string]interface{}{
				"value":       int(lvl.value),
				"string_name": lvl.value.String(),
			},
		)
	}
}

func captureLevelParsingTests(fs *capture.FixtureSet) {
	// Test parsing level from strings
	testCases := []struct {
		input    string
		expected log.Level
		valid    bool
	}{
		// Lowercase
		{"debug", log.DebugLevel, true},
		{"info", log.InfoLevel, true},
		{"warn", log.WarnLevel, true},
		{"error", log.ErrorLevel, true},
		{"fatal", log.FatalLevel, true},

		// Uppercase
		{"DEBUG", log.DebugLevel, true},
		{"INFO", log.InfoLevel, true},
		{"WARN", log.WarnLevel, true},
		{"ERROR", log.ErrorLevel, true},
		{"FATAL", log.FatalLevel, true},

		// Mixed case
		{"Debug", log.DebugLevel, true},
		{"Info", log.InfoLevel, true},
		{"Warn", log.WarnLevel, true},
		{"Error", log.ErrorLevel, true},
		{"Fatal", log.FatalLevel, true},

		// Warning alias
		{"warning", log.WarnLevel, true},
		{"WARNING", log.WarnLevel, true},

		// Invalid cases
		{"invalid", log.InfoLevel, false},
		{"", log.InfoLevel, false},
		{"123", log.InfoLevel, false},
		{"trace", log.InfoLevel, false},
	}

	for _, tc := range testCases {
		level, err := log.ParseLevel(tc.input)
		isValid := err == nil

		fs.AddTestWithCategory(fmt.Sprintf("parse_level_%s", sanitizeName(tc.input)), "unit",
			map[string]string{
				"input": tc.input,
			},
			map[string]interface{}{
				"level":    int(level),
				"is_valid": isValid,
			},
		)
	}
}

func captureLevelStringTests(fs *capture.FixtureSet) {
	// Test string representation of levels
	levels := []struct {
		value  log.Level
		expect string
	}{
		{log.DebugLevel, "debug"},
		{log.InfoLevel, "info"},
		{log.WarnLevel, "warn"},
		{log.ErrorLevel, "error"},
		{log.FatalLevel, "fatal"},
	}

	for _, lvl := range levels {
		fs.AddTestWithCategory(fmt.Sprintf("level_string_%s", lvl.expect), "unit",
			map[string]interface{}{
				"value": int(lvl.value),
			},
			map[string]string{
				"string": lvl.value.String(),
			},
		)
	}

	// Test custom level values
	customLevels := []log.Level{-8, -4, -2, 0, 2, 4, 6, 8, 10, 12, 16}
	for _, lvl := range customLevels {
		fs.AddTestWithCategory(fmt.Sprintf("level_string_custom_%d", lvl), "unit",
			map[string]interface{}{
				"value": int(lvl),
			},
			map[string]string{
				"string": lvl.String(),
			},
		)
	}
}

func captureLevelComparisonTests(fs *capture.FixtureSet) {
	// Test level ordering/comparison
	levels := []log.Level{
		log.DebugLevel,
		log.InfoLevel,
		log.WarnLevel,
		log.ErrorLevel,
		log.FatalLevel,
	}

	for i, lvl1 := range levels {
		for j, lvl2 := range levels {
			fs.AddTestWithCategory(
				fmt.Sprintf("level_compare_%s_vs_%s", lvl1.String(), lvl2.String()),
				"unit",
				map[string]interface{}{
					"level1":       int(lvl1),
					"level2":       int(lvl2),
					"level1_name":  lvl1.String(),
					"level2_name":  lvl2.String(),
				},
				map[string]interface{}{
					"less_than":       i < j,
					"greater_than":    i > j,
					"equal":           i == j,
					"level1_enabled_at_level2": lvl1 >= lvl2,
				},
			)
		}
	}
}

// sanitizeName converts a string to a valid test name
func sanitizeName(s string) string {
	if s == "" {
		return "empty"
	}
	result := ""
	for _, c := range s {
		if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') {
			result += string(c)
		} else {
			result += "_"
		}
	}
	return result
}
