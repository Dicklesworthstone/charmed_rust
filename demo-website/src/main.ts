import { examples, Example } from './examples';

// WASM module types (will be provided by charmed-wasm)
interface CharmedWasm {
  default: () => Promise<void>;
  newStyle: () => JsStyle;
  joinHorizontal: (pos: number, items: string[]) => string;
  joinVertical: (pos: number, items: string[]) => string;
  place: (width: number, height: number, hPos: number, vPos: number, content: string) => string;
  stringWidth: (s: string) => number;
  stringHeight: (s: string) => number;
  borderPresets: () => string[];
  version: () => string;
  isReady: () => boolean;
}

interface JsStyle {
  foreground(color: string): JsStyle;
  background(color: string): JsStyle;
  bold(): JsStyle;
  italic(): JsStyle;
  underline(): JsStyle;
  strikethrough(): JsStyle;
  faint(): JsStyle;
  reverse(): JsStyle;
  paddingAll(value: number): JsStyle;
  paddingVH(vertical: number, horizontal: number): JsStyle;
  padding(top: number, right: number, bottom: number, left: number): JsStyle;
  marginAll(value: number): JsStyle;
  marginVH(vertical: number, horizontal: number): JsStyle;
  margin(top: number, right: number, bottom: number, left: number): JsStyle;
  width(w: number): JsStyle;
  height(h: number): JsStyle;
  borderStyle(style: string): JsStyle;
  borderAll(): JsStyle;
  borderTop(): JsStyle;
  borderBottom(): JsStyle;
  borderLeft(): JsStyle;
  borderRight(): JsStyle;
  alignHorizontal(value: number): JsStyle;
  alignVertical(value: number): JsStyle;
  alignLeft(): JsStyle;
  alignCenter(): JsStyle;
  alignRight(): JsStyle;
  render(content: string): string;
  renderAnsi(content: string): string;
  copy(): JsStyle;
}

// Global state
let wasmModule: CharmedWasm | null = null;
let isInitialized = false;

// DOM elements
const codeInput = document.getElementById('code-input') as HTMLTextAreaElement;
const output = document.getElementById('output') as HTMLPreElement;
const errorDisplay = document.getElementById('error-display') as HTMLDivElement;
const exampleSelect = document.getElementById('example-select') as HTMLSelectElement;
const copyButton = document.getElementById('copy-output') as HTMLButtonElement;
const examplesGallery = document.getElementById('examples-gallery') as HTMLDivElement;
const heroTerminal = document.getElementById('hero-terminal') as HTMLDivElement;

// Show loading state
function showLoading() {
  output.innerHTML = '<span class="loading">Loading WASM module</span>';
}

// Show error
function showError(message: string) {
  errorDisplay.textContent = message;
  errorDisplay.classList.remove('hidden');
}

// Hide error
function hideError() {
  errorDisplay.classList.add('hidden');
}

// Initialize WASM module
async function initWasm(): Promise<boolean> {
  if (isInitialized) return true;

  showLoading();

  try {
    // Dynamic import of the WASM module
    // In development, this might be from node_modules or linked package
    // In production, it's from the built pkg
    const wasm = await import('charmed-wasm');
    await wasm.default();
    wasmModule = wasm as unknown as CharmedWasm;
    isInitialized = true;
    console.log('[charmed] WASM module loaded, version:', wasm.version());
    return true;
  } catch (e) {
    console.error('[charmed] Failed to load WASM module:', e);

    // Show helpful error message
    const errorMsg = e instanceof Error ? e.message : String(e);
    output.innerHTML = `<span style="color: #ff6b6b">Failed to load WASM module</span>

This demo requires the WASM package to be built first.
Run: <span style="color: #74b9ff">npm run build:wasm</span>

Error: ${errorMsg}`;
    return false;
  }
}

// Execute code and return result
function executeCode(code: string): string {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }

  // Create execution context with WASM functions
  const context = {
    newStyle: wasmModule.newStyle,
    joinHorizontal: wasmModule.joinHorizontal,
    joinVertical: wasmModule.joinVertical,
    place: wasmModule.place,
    stringWidth: wasmModule.stringWidth,
    stringHeight: wasmModule.stringHeight,
    borderPresets: wasmModule.borderPresets,
  };

  // Execute the code in the context
  const fn = new Function(...Object.keys(context), `return (${code})`);
  const result = fn(...Object.values(context));

  // Handle different return types
  if (typeof result === 'string') {
    return result;
  } else if (result && typeof result.render === 'function') {
    // If they returned a style without calling render
    return result.render('');
  } else if (result !== undefined) {
    return String(result);
  }

  return '';
}

// Update output preview
function updatePreview() {
  const code = codeInput.value.trim();

  if (!code) {
    output.textContent = '';
    hideError();
    return;
  }

  if (!isInitialized) {
    return;
  }

  try {
    const result = executeCode(code);
    output.innerHTML = result;
    hideError();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    showError(`Error: ${errorMsg}`);
    output.textContent = '';
  }
}

// Load example into editor
function loadExample(example: Example) {
  codeInput.value = example.code;
  updatePreview();

  // Scroll to editor
  document.getElementById('live-editor')?.scrollIntoView({ behavior: 'smooth' });
}

// Render example preview
function renderExamplePreview(example: Example): string {
  if (!isInitialized || !wasmModule) {
    return `<span style="color: #636e72">${example.name}</span>`;
  }

  try {
    return executeCode(example.code);
  } catch {
    return `<span style="color: #636e72">${example.name}</span>`;
  }
}

// Populate example select dropdown
function populateExampleSelect() {
  examples.forEach((example, index) => {
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = example.name;
    exampleSelect.appendChild(option);
  });
}

// Populate examples gallery
function populateExamplesGallery() {
  examplesGallery.innerHTML = '';

  examples.forEach((example) => {
    const card = document.createElement('div');
    card.className = 'example-card';
    card.innerHTML = `
      <div class="terminal">
        <div class="terminal-header">
          <span class="terminal-button red"></span>
          <span class="terminal-button yellow"></span>
          <span class="terminal-button green"></span>
        </div>
        <pre class="terminal-output">${renderExamplePreview(example)}</pre>
      </div>
      <div class="example-card-info">
        <div class="example-card-title">${example.name}</div>
        <div class="example-card-desc">${example.description}</div>
      </div>
    `;
    card.addEventListener('click', () => loadExample(example));
    examplesGallery.appendChild(card);
  });
}

// Render hero demo
function renderHeroDemo() {
  if (!isInitialized || !wasmModule) {
    heroTerminal.innerHTML = `
      <div class="terminal">
        <div class="terminal-header">
          <span class="terminal-button red"></span>
          <span class="terminal-button yellow"></span>
          <span class="terminal-button green"></span>
          <span class="terminal-title">demo</span>
        </div>
        <pre class="terminal-output loading">Loading WASM</pre>
      </div>
    `;
    return;
  }

  try {
    const newStyle = wasmModule.newStyle;
    const joinVertical = wasmModule.joinVertical;

    const title = newStyle()
      .foreground('#61dafb')
      .bold()
      .render('charmed_rust');

    const subtitle = newStyle()
      .foreground('#888888')
      .italic()
      .render('Terminal UI for the web');

    const box = newStyle()
      .borderStyle('rounded')
      .borderAll()
      .foreground('#58a6ff')
      .padding(1, 2, 1, 2)
      .render(joinVertical(0.5, [title, '', subtitle]));

    heroTerminal.innerHTML = `
      <div class="terminal">
        <div class="terminal-header">
          <span class="terminal-button red"></span>
          <span class="terminal-button yellow"></span>
          <span class="terminal-button green"></span>
          <span class="terminal-title">demo</span>
        </div>
        <pre class="terminal-output">${box}</pre>
      </div>
    `;
  } catch (e) {
    console.error('[charmed] Hero demo error:', e);
  }
}

// Copy output to clipboard
async function copyOutput() {
  const html = output.innerHTML;
  try {
    await navigator.clipboard.writeText(html);
    copyButton.textContent = 'Copied!';
    setTimeout(() => {
      copyButton.textContent = 'Copy HTML';
    }, 2000);
  } catch {
    showError('Failed to copy to clipboard');
  }
}

// Debounce helper
function debounce<T extends (...args: unknown[]) => unknown>(fn: T, delay: number): T {
  let timeoutId: ReturnType<typeof setTimeout>;
  return ((...args: unknown[]) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  }) as T;
}

// Initialize the application
async function main() {
  console.log('[charmed] Initializing demo website');

  // Set up event listeners
  codeInput.addEventListener('input', debounce(updatePreview, 300));

  exampleSelect.addEventListener('change', () => {
    const index = parseInt(exampleSelect.value, 10);
    if (!isNaN(index) && examples[index]) {
      loadExample(examples[index]);
    }
  });

  copyButton.addEventListener('click', copyOutput);

  // Populate UI elements
  populateExampleSelect();

  // Try to load WASM
  const success = await initWasm();

  if (success) {
    // Load initial example
    if (examples.length > 0) {
      codeInput.value = examples[0].code;
      updatePreview();
    }

    // Populate gallery with rendered previews
    populateExamplesGallery();

    // Render hero demo
    renderHeroDemo();
  } else {
    // Show static gallery
    populateExamplesGallery();
  }
}

// Run on DOM ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', main);
} else {
  main();
}
