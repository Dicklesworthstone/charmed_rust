// Example code snippets for the demo
export interface Example {
  name: string;
  description: string;
  code: string;
}

export const examples: Example[] = [
  {
    name: 'Basic Colors',
    description: 'Apply foreground and background colors',
    code: `newStyle()
  .foreground("#ff6b6b")
  .background("#2d2d2d")
  .paddingVH(1, 2)
  .render("Hello, World!")`,
  },
  {
    name: 'Text Formatting',
    description: 'Bold, italic, and underline text',
    code: `const bold = newStyle().bold().foreground("#61dafb").render("Bold ");
const italic = newStyle().italic().foreground("#98c379").render("Italic ");
const underline = newStyle().underline().foreground("#c678dd").render("Underline");
bold + italic + underline`,
  },
  {
    name: 'Rounded Border',
    description: 'Add a rounded border around content',
    code: `newStyle()
  .borderStyle("rounded")
  .borderAll()
  .foreground("#ffd93d")
  .paddingVH(1, 3)
  .render("Rounded Box")`,
  },
  {
    name: 'Double Border',
    description: 'Use double-line border style',
    code: `newStyle()
  .borderStyle("double")
  .borderAll()
  .foreground("#6c5ce7")
  .background("#1a1a2e")
  .paddingVH(1, 2)
  .render("Double Border")`,
  },
  {
    name: 'Vertical Layout',
    description: 'Stack elements vertically',
    code: `const header = newStyle()
  .background("#6c5ce7")
  .foreground("#ffffff")
  .paddingVH(0, 2)
  .width(20)
  .alignCenter()
  .render("Header");

const content = newStyle()
  .foreground("#74b9ff")
  .width(20)
  .render("Content here");

const footer = newStyle()
  .foreground("#636e72")
  .width(20)
  .render("Footer");

joinVertical(0, [header, content, footer])`,
  },
  {
    name: 'Horizontal Layout',
    description: 'Place elements side by side',
    code: `const left = newStyle()
  .background("#e17055")
  .foreground("#ffffff")
  .paddingAll(1)
  .render("Left");

const middle = newStyle()
  .background("#00b894")
  .foreground("#ffffff")
  .paddingAll(1)
  .render("Middle");

const right = newStyle()
  .background("#0984e3")
  .foreground("#ffffff")
  .paddingAll(1)
  .render("Right");

joinHorizontal(0.5, [left, middle, right])`,
  },
  {
    name: 'Centered Content',
    description: 'Center text within a fixed width',
    code: `newStyle()
  .width(30)
  .alignCenter()
  .foreground("#a29bfe")
  .borderStyle("rounded")
  .borderAll()
  .paddingAll(1)
  .render("Centered Text")`,
  },
  {
    name: 'Status Badge',
    description: 'Create a compact status indicator',
    code: `const success = newStyle()
  .background("#27ae60")
  .foreground("#ffffff")
  .bold()
  .paddingVH(0, 1)
  .render(" PASS ");

const fail = newStyle()
  .background("#e74c3c")
  .foreground("#ffffff")
  .bold()
  .paddingVH(0, 1)
  .render(" FAIL ");

const warn = newStyle()
  .background("#f39c12")
  .foreground("#000000")
  .bold()
  .paddingVH(0, 1)
  .render(" WARN ");

joinHorizontal(0, [success, " ", fail, " ", warn])`,
  },
  {
    name: 'Menu Item',
    description: 'Interactive-looking menu item',
    code: `const selected = newStyle()
  .background("#3498db")
  .foreground("#ffffff")
  .bold()
  .width(25)
  .paddingVH(0, 2)
  .render("> File");

const normal = newStyle()
  .foreground("#95a5a6")
  .width(25)
  .paddingVH(0, 2)
  .render("  Edit");

const disabled = newStyle()
  .foreground("#34495e")
  .faint()
  .width(25)
  .paddingVH(0, 2)
  .render("  View");

joinVertical(0, [selected, normal, disabled])`,
  },
  {
    name: 'Card Layout',
    description: 'Create a styled card component',
    code: `const title = newStyle()
  .foreground("#ecf0f1")
  .bold()
  .marginAll(0)
  .render("Card Title");

const body = newStyle()
  .foreground("#bdc3c7")
  .width(28)
  .render("This is the card body\\nwith multiple lines.");

const card = newStyle()
  .borderStyle("rounded")
  .borderAll()
  .foreground("#95a5a6")
  .paddingAll(1)
  .render(title + "\\n\\n" + body);

card`,
  },
];
