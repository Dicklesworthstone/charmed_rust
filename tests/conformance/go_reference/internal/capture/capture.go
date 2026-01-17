// Package capture provides utilities for capturing Go library behaviors
// as JSON fixtures for conformance testing.
package capture

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"
)

// FixtureMetadata contains metadata about the fixture set
type FixtureMetadata struct {
	Crate          string  `json:"crate"`
	GoVersion      string  `json:"go_version"`
	LibraryVersion string  `json:"library_version"`
	CapturedAt     string  `json:"captured_at"`
	Platform       *string `json:"platform,omitempty"`
	Notes          *string `json:"notes,omitempty"`
}

// TestFixture represents a single test case
type TestFixture struct {
	Name           string      `json:"name"`
	Category       *string     `json:"category,omitempty"`
	Input          interface{} `json:"input"`
	ExpectedOutput interface{} `json:"expected_output"`
	Notes          *string     `json:"notes,omitempty"`
	Tags           []string    `json:"tags,omitempty"`
	SkipReason     *string     `json:"skip_reason,omitempty"`
}

// FixtureSet is a complete set of fixtures for a crate
type FixtureSet struct {
	Metadata FixtureMetadata `json:"metadata"`
	Tests    []TestFixture   `json:"tests"`
}

// NewFixtureSet creates a new fixture set for a crate
func NewFixtureSet(crateName, libraryVersion string) *FixtureSet {
	return &FixtureSet{
		Metadata: FixtureMetadata{
			Crate:          crateName,
			GoVersion:      "1.24",
			LibraryVersion: libraryVersion,
			CapturedAt:     time.Now().UTC().Format(time.RFC3339),
		},
		Tests: make([]TestFixture, 0),
	}
}

// AddTest adds a test fixture to the set
func (fs *FixtureSet) AddTest(name string, input, output interface{}) {
	fs.Tests = append(fs.Tests, TestFixture{
		Name:           name,
		Input:          input,
		ExpectedOutput: output,
	})
}

// AddTestWithCategory adds a test fixture with a category
func (fs *FixtureSet) AddTestWithCategory(name, category string, input, output interface{}) {
	cat := category
	fs.Tests = append(fs.Tests, TestFixture{
		Name:           name,
		Category:       &cat,
		Input:          input,
		ExpectedOutput: output,
	})
}

// AddTestWithNotes adds a test fixture with notes
func (fs *FixtureSet) AddTestWithNotes(name string, input, output interface{}, notes string) {
	n := notes
	fs.Tests = append(fs.Tests, TestFixture{
		Name:           name,
		Input:          input,
		ExpectedOutput: output,
		Notes:          &n,
	})
}

// WriteToFile writes the fixture set to a JSON file
func (fs *FixtureSet) WriteToFile(outputDir string) error {
	filename := filepath.Join(outputDir, fs.Metadata.Crate+".json")
	data, err := json.MarshalIndent(fs, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal fixtures: %w", err)
	}

	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return fmt.Errorf("failed to create output directory: %w", err)
	}

	if err := os.WriteFile(filename, data, 0644); err != nil {
		return fmt.Errorf("failed to write fixture file: %w", err)
	}

	fmt.Printf("Wrote %d tests to %s\n", len(fs.Tests), filename)
	return nil
}

// Ptr is a helper to create a pointer to a string
func Ptr(s string) *string {
	return &s
}

// SpringInput represents input for spring physics tests
type SpringInput struct {
	Frequency      float64 `json:"frequency"`
	Damping        float64 `json:"damping"`
	CurrentPos     float64 `json:"current_pos"`
	TargetPos      float64 `json:"target_pos"`
	Velocity       float64 `json:"velocity"`
	DeltaTime      float64 `json:"delta_time"`
}

// SpringOutput represents output from spring physics tests
type SpringOutput struct {
	NewPos      float64 `json:"new_pos"`
	NewVelocity float64 `json:"new_velocity"`
}

// ProjectileInput represents input for projectile tests
type ProjectileInput struct {
	X         float64 `json:"x"`
	Y         float64 `json:"y"`
	Z         float64 `json:"z"`
	VelX      float64 `json:"vel_x"`
	VelY      float64 `json:"vel_y"`
	VelZ      float64 `json:"vel_z"`
	Gravity   float64 `json:"gravity"`
	DeltaTime float64 `json:"delta_time"`
}

// ProjectileOutput represents output from projectile tests
type ProjectileOutput struct {
	X    float64 `json:"x"`
	Y    float64 `json:"y"`
	Z    float64 `json:"z"`
	VelX float64 `json:"vel_x"`
	VelY float64 `json:"vel_y"`
	VelZ float64 `json:"vel_z"`
}

// StyleInput represents input for style rendering tests
type StyleInput struct {
	Foreground   *string `json:"foreground,omitempty"`
	Background   *string `json:"background,omitempty"`
	Bold         bool    `json:"bold"`
	Italic       bool    `json:"italic"`
	Underline    bool    `json:"underline"`
	Strikethrough bool   `json:"strikethrough"`
	Faint        bool    `json:"faint"`
	Blink        bool    `json:"blink"`
	Reverse      bool    `json:"reverse"`
	Text         string  `json:"text"`
	Width        int     `json:"width,omitempty"`
	Height       int     `json:"height,omitempty"`
	Padding      []int   `json:"padding,omitempty"`
	Margin       []int   `json:"margin,omitempty"`
}

// StyleOutput represents output from style rendering tests
type StyleOutput struct {
	Rendered string `json:"rendered"`
	Width    int    `json:"width"`
	Height   int    `json:"height"`
}

// BorderInput represents input for border rendering tests
type BorderInput struct {
	BorderType string  `json:"border_type"`
	Text       string  `json:"text"`
	Foreground *string `json:"foreground,omitempty"`
	Background *string `json:"background,omitempty"`
	Width      int     `json:"width,omitempty"`
}

// BorderOutput represents output from border rendering tests
type BorderOutput struct {
	Rendered string `json:"rendered"`
}
