// Basic spacing variations
Empty: *    *
No spaces:*bold*
Left space: *bold*
Right space:*bold *
Both spaces: * bold *

// Multiple spacing variations
Too many spaces:  *  bold  *  
Tab spacing:	*	bold	*
Mixed spacing: * bold*   *bold * *  bold  *

// Line breaks with strong elements
Line
*breaks
inside* strong and text *across
multiple
lines* with content

// Complex nesting with inconsistent spacing
Nested: *bold with _emphasis_ inside* vs * bold with _nested
emphasis_ across lines*

// Strong elements in structural elements
- List item with*no space*around strong
- Another item with * excessive    spacing *

// Different contexts
"Quotation with *strong * element"
#text()[Text function with *strong* and * bad 
spacing *]

// Mixed languages with spacing issues
English and *中文* mixed with*不同*spacing patterns
*中文与English* mixed *在同一个*strong element

// Adjacent strong elements with varying spacing
*one**two* *three*   *four*     *five*

// Comment with strong
// This is a *comment* with * strong * elements

// Chinese with various spacing patterns
中文测试：*无空格* text，前空格 *有空格*，后空格 *有空格* text，*内部 空格*，
行
*跨越
换行* 以及 *嵌套 _强调_* *相邻*  *元素*
