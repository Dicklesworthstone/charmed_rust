// Huh capture program - captures form component behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	"github.com/charmbracelet/huh"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("huh", "0.6.0")

	// Capture input field tests
	captureInputFieldTests(fixtures)

	// Capture text field tests
	captureTextFieldTests(fixtures)

	// Capture select field tests
	captureSelectFieldTests(fixtures)

	// Capture multi-select field tests
	captureMultiSelectFieldTests(fixtures)

	// Capture confirm field tests
	captureConfirmFieldTests(fixtures)

	// Capture note field tests
	captureNoteFieldTests(fixtures)

	// Capture validation tests
	captureValidationTests(fixtures)

	// Capture theme tests
	captureThemeTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureInputFieldTests(fs *capture.FixtureSet) {
	// Test 1: Basic input field
	{
		var value string
		input := huh.NewInput().
			Title("Enter name").
			Value(&value)

		fs.AddTestWithCategory("input_basic", "unit",
			map[string]interface{}{
				"title": "Enter name",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "input",
			},
		)
	}

	// Test 2: Input with placeholder
	{
		var value string
		input := huh.NewInput().
			Title("Email").
			Placeholder("user@example.com").
			Value(&value)
		_ = input

		fs.AddTestWithCategory("input_placeholder", "unit",
			map[string]interface{}{
				"title":       "Email",
				"placeholder": "user@example.com",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "input",
			},
		)
	}

	// Test 3: Input with character limit
	{
		var value string
		input := huh.NewInput().
			Title("Username").
			CharLimit(20).
			Value(&value)
		_ = input

		fs.AddTestWithCategory("input_char_limit", "unit",
			map[string]interface{}{
				"title":      "Username",
				"char_limit": 20,
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "input",
			},
		)
	}

	// Test 4: Input with description
	{
		var value string
		input := huh.NewInput().
			Title("Password").
			Description("Must be at least 8 characters").
			Value(&value)
		_ = input

		fs.AddTestWithCategory("input_description", "unit",
			map[string]interface{}{
				"title":       "Password",
				"description": "Must be at least 8 characters",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "input",
			},
		)
	}

	// Test 5: Password input (echoMode)
	{
		var value string
		input := huh.NewInput().
			Title("Password").
			EchoMode(huh.EchoModePassword).
			Value(&value)
		_ = input

		fs.AddTestWithCategory("input_password", "unit",
			map[string]interface{}{
				"title":     "Password",
				"echo_mode": "password",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "input",
				"echo_mode":     int(huh.EchoModePassword),
			},
		)
	}

	// Test 6: Input with initial value
	{
		value := "initial"
		input := huh.NewInput().
			Title("Name").
			Value(&value)
		_ = input

		fs.AddTestWithCategory("input_initial_value", "unit",
			map[string]interface{}{
				"title":         "Name",
				"initial_value": "initial",
			},
			map[string]interface{}{
				"value":      value,
				"field_type": "input",
			},
		)
	}
}

func captureTextFieldTests(fs *capture.FixtureSet) {
	// Test 1: Basic text area
	{
		var value string
		text := huh.NewText().
			Title("Description").
			Value(&value)
		_ = text

		fs.AddTestWithCategory("text_basic", "unit",
			map[string]interface{}{
				"title": "Description",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "text",
			},
		)
	}

	// Test 2: Text area with lines
	{
		var value string
		text := huh.NewText().
			Title("Bio").
			Lines(5).
			Value(&value)
		_ = text

		fs.AddTestWithCategory("text_with_lines", "unit",
			map[string]interface{}{
				"title": "Bio",
				"lines": 5,
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "text",
			},
		)
	}

	// Test 3: Text area with placeholder
	{
		var value string
		text := huh.NewText().
			Title("Notes").
			Placeholder("Enter your notes...").
			Value(&value)
		_ = text

		fs.AddTestWithCategory("text_placeholder", "unit",
			map[string]interface{}{
				"title":       "Notes",
				"placeholder": "Enter your notes...",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "text",
			},
		)
	}

	// Test 4: Text area with char limit
	{
		var value string
		text := huh.NewText().
			Title("Message").
			CharLimit(500).
			Value(&value)
		_ = text

		fs.AddTestWithCategory("text_char_limit", "unit",
			map[string]interface{}{
				"title":      "Message",
				"char_limit": 500,
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "text",
			},
		)
	}
}

func captureSelectFieldTests(fs *capture.FixtureSet) {
	// Test 1: Basic select
	{
		var value string
		sel := huh.NewSelect[string]().
			Title("Choose an option").
			Options(
				huh.NewOption("Option A", "a"),
				huh.NewOption("Option B", "b"),
				huh.NewOption("Option C", "c"),
			).
			Value(&value)
		_ = sel

		fs.AddTestWithCategory("select_basic", "unit",
			map[string]interface{}{
				"title":   "Choose an option",
				"options": []string{"a", "b", "c"},
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "select",
			},
		)
	}

	// Test 2: Select with description
	{
		var value string
		sel := huh.NewSelect[string]().
			Title("Select Color").
			Description("Choose your favorite").
			Options(
				huh.NewOption("Red", "red"),
				huh.NewOption("Green", "green"),
				huh.NewOption("Blue", "blue"),
			).
			Value(&value)
		_ = sel

		fs.AddTestWithCategory("select_description", "unit",
			map[string]interface{}{
				"title":       "Select Color",
				"description": "Choose your favorite",
				"options":     []string{"red", "green", "blue"},
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "select",
			},
		)
	}

	// Test 3: Select with height limit
	{
		var value string
		sel := huh.NewSelect[string]().
			Title("Choose").
			Height(5).
			Options(
				huh.NewOption("1", "1"),
				huh.NewOption("2", "2"),
				huh.NewOption("3", "3"),
				huh.NewOption("4", "4"),
				huh.NewOption("5", "5"),
				huh.NewOption("6", "6"),
				huh.NewOption("7", "7"),
			).
			Value(&value)
		_ = sel

		fs.AddTestWithCategory("select_height", "unit",
			map[string]interface{}{
				"title":  "Choose",
				"height": 5,
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "select",
			},
		)
	}

	// Test 4: Integer select
	{
		var value int
		sel := huh.NewSelect[int]().
			Title("Choose number").
			Options(
				huh.NewOption("One", 1),
				huh.NewOption("Two", 2),
				huh.NewOption("Three", 3),
			).
			Value(&value)
		_ = sel

		fs.AddTestWithCategory("select_int", "unit",
			map[string]interface{}{
				"title":   "Choose number",
				"options": []int{1, 2, 3},
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "select",
			},
		)
	}
}

func captureMultiSelectFieldTests(fs *capture.FixtureSet) {
	// Test 1: Basic multi-select
	{
		var values []string
		ms := huh.NewMultiSelect[string]().
			Title("Select items").
			Options(
				huh.NewOption("Item A", "a"),
				huh.NewOption("Item B", "b"),
				huh.NewOption("Item C", "c"),
			).
			Value(&values)
		_ = ms

		fs.AddTestWithCategory("multiselect_basic", "unit",
			map[string]interface{}{
				"title":   "Select items",
				"options": []string{"a", "b", "c"},
			},
			map[string]interface{}{
				"initial_value": values,
				"field_type":    "multiselect",
			},
		)
	}

	// Test 2: Multi-select with limit
	{
		var values []string
		ms := huh.NewMultiSelect[string]().
			Title("Select up to 2").
			Limit(2).
			Options(
				huh.NewOption("X", "x"),
				huh.NewOption("Y", "y"),
				huh.NewOption("Z", "z"),
			).
			Value(&values)
		_ = ms

		fs.AddTestWithCategory("multiselect_limit", "unit",
			map[string]interface{}{
				"title":   "Select up to 2",
				"limit":   2,
				"options": []string{"x", "y", "z"},
			},
			map[string]interface{}{
				"initial_value": values,
				"field_type":    "multiselect",
			},
		)
	}

	// Test 3: Multi-select with description
	{
		var values []string
		ms := huh.NewMultiSelect[string]().
			Title("Select features").
			Description("Choose multiple").
			Options(
				huh.NewOption("Feature 1", "f1"),
				huh.NewOption("Feature 2", "f2"),
				huh.NewOption("Feature 3", "f3"),
			).
			Value(&values)
		_ = ms

		fs.AddTestWithCategory("multiselect_description", "unit",
			map[string]interface{}{
				"title":       "Select features",
				"description": "Choose multiple",
				"options":     []string{"f1", "f2", "f3"},
			},
			map[string]interface{}{
				"initial_value": values,
				"field_type":    "multiselect",
			},
		)
	}

	// Test 4: Multi-select with pre-selected values
	{
		values := []string{"a"}
		ms := huh.NewMultiSelect[string]().
			Title("Modify selection").
			Options(
				huh.NewOption("A", "a").Selected(true),
				huh.NewOption("B", "b"),
				huh.NewOption("C", "c"),
			).
			Value(&values)
		_ = ms

		fs.AddTestWithCategory("multiselect_preselected", "unit",
			map[string]interface{}{
				"title":       "Modify selection",
				"preselected": []string{"a"},
			},
			map[string]interface{}{
				"initial_value": values,
				"field_type":    "multiselect",
			},
		)
	}
}

func captureConfirmFieldTests(fs *capture.FixtureSet) {
	// Test 1: Basic confirm
	{
		var value bool
		conf := huh.NewConfirm().
			Title("Continue?").
			Value(&value)
		_ = conf

		fs.AddTestWithCategory("confirm_basic", "unit",
			map[string]interface{}{
				"title": "Continue?",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "confirm",
			},
		)
	}

	// Test 2: Confirm with description
	{
		var value bool
		conf := huh.NewConfirm().
			Title("Delete file?").
			Description("This action cannot be undone").
			Value(&value)
		_ = conf

		fs.AddTestWithCategory("confirm_description", "unit",
			map[string]interface{}{
				"title":       "Delete file?",
				"description": "This action cannot be undone",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "confirm",
			},
		)
	}

	// Test 3: Confirm with affirmative/negative labels
	{
		var value bool
		conf := huh.NewConfirm().
			Title("Save changes?").
			Affirmative("Yes!").
			Negative("No").
			Value(&value)
		_ = conf

		fs.AddTestWithCategory("confirm_labels", "unit",
			map[string]interface{}{
				"title":       "Save changes?",
				"affirmative": "Yes!",
				"negative":    "No",
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "confirm",
			},
		)
	}

	// Test 4: Confirm with default true
	{
		value := true
		conf := huh.NewConfirm().
			Title("Enable feature?").
			Value(&value)
		_ = conf

		fs.AddTestWithCategory("confirm_default_true", "unit",
			map[string]interface{}{
				"title":   "Enable feature?",
				"default": true,
			},
			map[string]interface{}{
				"initial_value": value,
				"field_type":    "confirm",
			},
		)
	}
}

func captureNoteFieldTests(fs *capture.FixtureSet) {
	// Test 1: Basic note
	{
		note := huh.NewNote().
			Title("Important").
			Description("Please read carefully")
		_ = note

		fs.AddTestWithCategory("note_basic", "unit",
			map[string]interface{}{
				"title":       "Important",
				"description": "Please read carefully",
			},
			map[string]interface{}{
				"field_type": "note",
			},
		)
	}

	// Test 2: Note title only
	{
		note := huh.NewNote().
			Title("Welcome")
		_ = note

		fs.AddTestWithCategory("note_title_only", "unit",
			map[string]interface{}{
				"title": "Welcome",
			},
			map[string]interface{}{
				"field_type": "note",
			},
		)
	}

	// Test 3: Note with next label
	{
		note := huh.NewNote().
			Title("Step 1").
			Description("First step description").
			Next(true).
			NextLabel("Continue")
		_ = note

		fs.AddTestWithCategory("note_next", "unit",
			map[string]interface{}{
				"title":      "Step 1",
				"next":       true,
				"next_label": "Continue",
			},
			map[string]interface{}{
				"field_type": "note",
			},
		)
	}
}

func captureValidationTests(fs *capture.FixtureSet) {
	// Test 1: Required validation
	{
		var value string
		input := huh.NewInput().
			Title("Name").
			Validate(func(s string) error {
				if s == "" {
					return fmt.Errorf("name is required")
				}
				return nil
			}).
			Value(&value)
		_ = input

		// Test with empty value
		err := input.Validate(value)
		errStr := ""
		if err != nil {
			errStr = err.Error()
		}

		fs.AddTestWithCategory("validation_required", "unit",
			map[string]interface{}{
				"title":      "Name",
				"test_value": "",
			},
			map[string]interface{}{
				"has_error":     err != nil,
				"error_message": errStr,
			},
		)
	}

	// Test 2: Minimum length validation
	{
		var value string
		input := huh.NewInput().
			Title("Password").
			Validate(func(s string) error {
				if len(s) < 8 {
					return fmt.Errorf("password must be at least 8 characters")
				}
				return nil
			}).
			Value(&value)
		_ = input

		// Test with short password
		value = "short"
		err := input.Validate(value)
		errStr := ""
		if err != nil {
			errStr = err.Error()
		}

		fs.AddTestWithCategory("validation_min_length", "unit",
			map[string]interface{}{
				"title":      "Password",
				"min_length": 8,
				"test_value": "short",
			},
			map[string]interface{}{
				"has_error":     err != nil,
				"error_message": errStr,
			},
		)
	}

	// Test 3: Passing validation
	{
		var value string
		input := huh.NewInput().
			Title("Email").
			Validate(func(s string) error {
				if s == "" {
					return fmt.Errorf("email required")
				}
				return nil
			}).
			Value(&value)
		_ = input

		// Test with valid value
		value = "user@example.com"
		err := input.Validate(value)

		fs.AddTestWithCategory("validation_pass", "unit",
			map[string]interface{}{
				"title":      "Email",
				"test_value": "user@example.com",
			},
			map[string]interface{}{
				"has_error": err != nil,
			},
		)
	}
}

func captureThemeTests(fs *capture.FixtureSet) {
	// Capture available theme information
	themes := []struct {
		name  string
		theme *huh.Theme
	}{
		{"base", huh.ThemeBase()},
		{"charm", huh.ThemeCharm()},
		{"dracula", huh.ThemeDracula()},
		{"catppuccin", huh.ThemeCatppuccin()},
	}

	for _, t := range themes {
		fs.AddTestWithCategory(fmt.Sprintf("theme_%s", t.name), "unit",
			map[string]interface{}{
				"theme_name": t.name,
			},
			map[string]interface{}{
				"theme_available": t.theme != nil,
			},
		)
	}

	// Test form with theme
	{
		var value string
		form := huh.NewForm(
			huh.NewGroup(
				huh.NewInput().
					Title("Name").
					Value(&value),
			),
		).WithTheme(huh.ThemeCharm())
		_ = form

		fs.AddTestWithCategory("form_with_theme", "unit",
			map[string]interface{}{
				"theme": "charm",
			},
			map[string]interface{}{
				"form_created": form != nil,
			},
		)
	}
}
