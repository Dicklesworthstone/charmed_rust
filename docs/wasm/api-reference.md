# API Reference

Complete API documentation for charmed-wasm.

## Module Functions

### `init()`

Initialize the WASM module. Must be called before using any other functions.

```typescript
import init from '@charmed/wasm';

await init();
```

**Returns**: `Promise<void>`

### `newStyle()`

Create a new style builder.

```typescript
import { newStyle } from '@charmed/wasm';

const style = newStyle();
```

**Returns**: `JsStyle` - A chainable style builder

### `version()`

Get the charmed-wasm version string.

```typescript
import { version } from '@charmed/wasm';

console.log(version()); // "0.1.0"
```

**Returns**: `string`

### `isReady()`

Check if the WASM module is properly initialized.

```typescript
import { isReady } from '@charmed/wasm';

if (isReady()) {
  // Safe to use styling functions
}
```

**Returns**: `boolean`

---

## JsStyle

The main style builder class. All methods return a new `JsStyle` for chaining.

### Color Methods

#### `.foreground(color: string)`

Set the text (foreground) color.

```typescript
newStyle().foreground('#ff0000')  // Hex color
newStyle().foreground('196')      // ANSI 256 color
```

#### `.background(color: string)`

Set the background color.

```typescript
newStyle().background('#1a1a2e')
```

### Text Formatting

#### `.bold()`

Enable bold text.

```typescript
newStyle().bold()
```

#### `.italic()`

Enable italic text.

```typescript
newStyle().italic()
```

#### `.underline()`

Enable underlined text.

```typescript
newStyle().underline()
```

#### `.strikethrough()`

Enable strikethrough text.

```typescript
newStyle().strikethrough()
```

#### `.faint()`

Enable faint/dim text.

```typescript
newStyle().faint()
```

#### `.reverse()`

Swap foreground and background colors.

```typescript
newStyle().reverse()
```

### Padding Methods

#### `.paddingAll(value: number)`

Set padding on all sides.

```typescript
newStyle().paddingAll(2) // 2 units on all sides
```

#### `.paddingVH(vertical: number, horizontal: number)`

Set padding with vertical and horizontal values.

```typescript
newStyle().paddingVH(1, 2) // 1 top/bottom, 2 left/right
```

#### `.padding(top: number, right: number, bottom: number, left: number)`

Set padding for each side individually.

```typescript
newStyle().padding(1, 2, 1, 2) // CSS-like: top, right, bottom, left
```

### Margin Methods

#### `.marginAll(value: number)`

Set margin on all sides.

```typescript
newStyle().marginAll(1)
```

#### `.marginVH(vertical: number, horizontal: number)`

Set margin with vertical and horizontal values.

```typescript
newStyle().marginVH(1, 2)
```

#### `.margin(top: number, right: number, bottom: number, left: number)`

Set margin for each side individually.

```typescript
newStyle().margin(0, 1, 0, 1)
```

### Dimension Methods

#### `.width(w: number)`

Set a fixed width for the content.

```typescript
newStyle().width(40)
```

#### `.height(h: number)`

Set a fixed height for the content.

```typescript
newStyle().height(5)
```

### Border Methods

#### `.borderStyle(style: string)`

Set the border style. Available presets:

- `"normal"` - Single-line box drawing
- `"rounded"` - Rounded corners
- `"thick"` - Thick lines
- `"double"` - Double lines
- `"hidden"` - Invisible (space only)
- `"ascii"` - ASCII characters (+, -, |)

```typescript
newStyle().borderStyle('rounded')
```

#### `.borderAll()`

Enable borders on all sides.

```typescript
newStyle().borderStyle('rounded').borderAll()
```

#### `.borderTop()`

Enable top border only.

```typescript
newStyle().borderStyle('normal').borderTop()
```

#### `.borderBottom()`

Enable bottom border only.

```typescript
newStyle().borderStyle('normal').borderBottom()
```

#### `.borderLeft()`

Enable left border only.

```typescript
newStyle().borderStyle('normal').borderLeft()
```

#### `.borderRight()`

Enable right border only.

```typescript
newStyle().borderStyle('normal').borderRight()
```

### Alignment Methods

#### `.alignHorizontal(value: number)`

Set horizontal alignment using a position value:
- `0.0` = left
- `0.5` = center
- `1.0` = right

```typescript
newStyle().width(30).alignHorizontal(0.5) // Center
```

#### `.alignVertical(value: number)`

Set vertical alignment using a position value:
- `0.0` = top
- `0.5` = middle
- `1.0` = bottom

```typescript
newStyle().height(5).alignVertical(0.5) // Middle
```

#### `.alignLeft()`

Shorthand for left alignment.

```typescript
newStyle().alignLeft()
```

#### `.alignCenter()`

Shorthand for center alignment.

```typescript
newStyle().alignCenter()
```

#### `.alignRight()`

Shorthand for right alignment.

```typescript
newStyle().alignRight()
```

### Render Methods

#### `.render(content: string)`

Render content with the style applied. Returns HTML with inline styles.

```typescript
const html = newStyle().bold().render('Hello');
// Returns styled HTML string
```

**Returns**: `string` - HTML string

#### `.renderAnsi(content: string)`

Render content with ANSI escape sequences. Useful for terminal-like displays.

```typescript
const ansi = newStyle().bold().renderAnsi('Hello');
// Returns string with ANSI escape codes
```

**Returns**: `string` - ANSI-escaped string

### Utility Methods

#### `.copy()`

Create a copy of this style.

```typescript
const baseStyle = newStyle().bold();
const redStyle = baseStyle.copy().foreground('#ff0000');
const blueStyle = baseStyle.copy().foreground('#0000ff');
```

**Returns**: `JsStyle`

---

## Layout Functions

### `joinHorizontal(position: number, items: string[])`

Join multiple strings horizontally (side by side).

**Parameters**:
- `position` - Vertical alignment (0.0 = top, 0.5 = center, 1.0 = bottom)
- `items` - Array of strings to join

```typescript
import { joinHorizontal, newStyle } from '@charmed/wasm';

const left = newStyle().render('Left');
const right = newStyle().render('Right');
const result = joinHorizontal(0.5, [left, right]);
```

**Returns**: `string`

### `joinVertical(position: number, items: string[])`

Join multiple strings vertically (stacked).

**Parameters**:
- `position` - Horizontal alignment (0.0 = left, 0.5 = center, 1.0 = right)
- `items` - Array of strings to join

```typescript
import { joinVertical, newStyle } from '@charmed/wasm';

const top = newStyle().render('Top');
const bottom = newStyle().render('Bottom');
const result = joinVertical(0.5, [top, bottom]);
```

**Returns**: `string`

### `place(width: number, height: number, hPos: number, vPos: number, content: string)`

Place content at a position within a container of specified dimensions.

**Parameters**:
- `width` - Container width
- `height` - Container height
- `hPos` - Horizontal position (0.0 = left, 0.5 = center, 1.0 = right)
- `vPos` - Vertical position (0.0 = top, 0.5 = center, 1.0 = bottom)
- `content` - The content to place

```typescript
import { place } from '@charmed/wasm';

const centered = place(40, 10, 0.5, 0.5, 'Centered!');
```

**Returns**: `string`

---

## Utility Functions

### `stringWidth(s: string)`

Get the visible width of a string (excluding escape codes).

```typescript
import { stringWidth } from '@charmed/wasm';

const width = stringWidth('Hello'); // 5
```

**Returns**: `number`

### `stringHeight(s: string)`

Get the height (number of lines) of a string.

```typescript
import { stringHeight } from '@charmed/wasm';

const height = stringHeight('Line 1\nLine 2\nLine 3'); // 3
```

**Returns**: `number`

### `borderPresets()`

Get a list of available border preset names.

```typescript
import { borderPresets } from '@charmed/wasm';

const presets = borderPresets();
// ['normal', 'rounded', 'thick', 'double', 'hidden', 'ascii']
```

**Returns**: `string[]`

---

## JsColor

Color utility class.

### `JsColor.fromHex(hex: string)`

Create a color from a hex string.

```typescript
import { JsColor } from '@charmed/wasm';

const color = JsColor.fromHex('#ff0000');
```

### `JsColor.fromRgb(r: number, g: number, b: number)`

Create a color from RGB values.

```typescript
const color = JsColor.fromRgb(255, 0, 0);
```

### `JsColor.fromAnsi(code: number)`

Create a color from an ANSI 256 color code.

```typescript
const color = JsColor.fromAnsi(196); // Bright red
```

### `.toHex()`

Get the hex representation of the color.

```typescript
const hex = color.toHex(); // '#ff0000'
```

**Returns**: `string`
