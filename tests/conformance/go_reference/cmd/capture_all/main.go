// Capture all - orchestrates running all capture programs
package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"time"
)

func main() {
	outputDir := flag.String("output", "../../../fixtures/go_outputs", "Output directory for fixtures")
	verbose := flag.Bool("verbose", false, "Verbose output")
	flag.Parse()

	// Get the absolute path of the output directory
	absOutput, err := filepath.Abs(*outputDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error resolving output path: %v\n", err)
		os.Exit(1)
	}

	// Ensure output directory exists
	if err := os.MkdirAll(absOutput, 0755); err != nil {
		fmt.Fprintf(os.Stderr, "Error creating output directory: %v\n", err)
		os.Exit(1)
	}

	// List of capture programs to run
	programs := []string{
		"harmonica",
		"lipgloss",
		"bubbletea",
		"bubbles",
		"log",
		"glamour",
		"huh",
		"wish",
	}

	fmt.Println("=== Charmed Rust Conformance Capture ===")
	fmt.Printf("Output directory: %s\n", absOutput)
	fmt.Printf("Running %d capture programs\n\n", len(programs))

	startTime := time.Now()
	successes := 0
	failures := 0

	for i, prog := range programs {
		fmt.Printf("[%d/%d] Capturing %s...", i+1, len(programs), prog)

		// Build the program first
		buildCmd := exec.Command("go", "build", "-o", fmt.Sprintf("/tmp/capture_%s", prog), fmt.Sprintf("./cmd/%s", prog))
		buildCmd.Dir = filepath.Join(filepath.Dir(os.Args[0]), "..")
		if *verbose {
			buildCmd.Stdout = os.Stdout
			buildCmd.Stderr = os.Stderr
		}

		if err := buildCmd.Run(); err != nil {
			fmt.Printf(" BUILD FAILED: %v\n", err)
			failures++
			continue
		}

		// Run the capture program
		runCmd := exec.Command(fmt.Sprintf("/tmp/capture_%s", prog), "-output", absOutput)
		if *verbose {
			runCmd.Stdout = os.Stdout
			runCmd.Stderr = os.Stderr
		}

		if err := runCmd.Run(); err != nil {
			fmt.Printf(" RUN FAILED: %v\n", err)
			failures++
			continue
		}

		// Verify output file exists
		outputFile := filepath.Join(absOutput, prog+".json")
		if _, err := os.Stat(outputFile); err != nil {
			fmt.Printf(" NO OUTPUT FILE\n")
			failures++
			continue
		}

		info, _ := os.Stat(outputFile)
		fmt.Printf(" OK (%d bytes)\n", info.Size())
		successes++
	}

	elapsed := time.Since(startTime)
	fmt.Println()
	fmt.Println("=== Summary ===")
	fmt.Printf("Successful: %d/%d\n", successes, len(programs))
	fmt.Printf("Failed: %d/%d\n", failures, len(programs))
	fmt.Printf("Time: %v\n", elapsed)

	if failures > 0 {
		os.Exit(1)
	}
}
