# LOTUS design system

## Shell surfaces

Use the three roles below. Do not introduce another page, panel, or chrome
background token.

  | Role                    | Utility           | Background                        | Radius               | Separation                       |
  | ---                     | ---               | ---                               | ---                  | ---                              |
  | Page                    | `bg-shell-page`   | `--shell-page-bg` (`--bg`)        | none                 | none                             |
  | Raised panel/card/table | `bg-shell-raised` | `--shell-raised-bg` (`--surface`) | `rounded-xl` (12 px) | `border-shell-border`, no shadow |
  | Header/footer/metadata  | `bg-shell-chrome` | `--shell-chrome-bg` (`--surface`) | header/meta only     | `border-shell-border`, no shadow |

Controls keep their existing 8 px radius and may use the existing subtle shadow.
Shell roles never combine a border and a shadow. A shell has no background when
it is an unframed layout wrapper or an inner region of a raised panel; do not
stack one raised surface inside another merely to create spacing. Tone-colored
notice backgrounds are semantic status surfaces, not shell levels.

Tokens are defined only in `tailwind/styles.css`. Tailwind classes remain inline
in RSX; do not wrap styling rules in Rust components.

## Semantics and RDFa

Use native HTML first. Every page has one `h1`, followed by ordered `h2`/`h3`
headings; `header`, `nav`, `main`, `section`, `footer`, `table`, `time`, labels,
and native controls are preferred over ARIA replacements. Visible text is the
accessible name; use `aria-label` only for icon-only controls. Stable automation
hooks are `data-lotus-id` for domain entities and `data-segmented-value` for
segmented controls; existing `data-mcp-*` hooks remain the schema-oriented form
API.

Declare `vocab="https://schema.org/"` at the domain-content scope. Use canonical
Wikidata resources for QIDs and established schema.org terms only:

  | Entity/data         | RDFa                                                                          |
  | ---                 | ---                                                                           |
  | Result collection   | `typeof="ItemList"`, `property="numberOfItems"` with `content`                |
  | Compound row        | `typeof="ChemicalEntity"`, `about="https://www.wikidata.org/entity/{QID}"`    |
  | Compound attributes | `schema:name`; Wikidata P233 SMILES, P235 InChIKey, P2062 mass, P274 formula  |
  | Taxon               | Wikidata P171, `typeof="Taxon"`, Wikidata entity `resource`                   |
  | Source article      | Wikidata P248, `typeof="ScholarlyArticle"`, Wikidata entity `resource`        |
  | DOI                 | `property="identifier"`, `resource="https://doi.org/{DOI}"`                   |
  | Publication year    | `time[property="datePublished"]` with `datetime`                              |

Do not annotate decorative controls, labels, or layout wrappers. Do not invent
vocabulary terms. RDFa custom attributes in Dioxus RSX must use quoted HTML
names, for example `"property": "name"`.
