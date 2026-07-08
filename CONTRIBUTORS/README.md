# CONTRIBUTORS

This directory contains metadata about the contributors to this project.

Each contributor has their own YAML file containing structured information about their roles, expertise, contact preferences, and an optional Markdown profile.

## Directory Structure

```text
CONTRIBUTORS/
├── README.md
├── l0rdcycl0p.yml
├── alice.yml
└── bob.yml
```

Each contributor file **must** be named after the contributor's unique ID.

Example:

```text
l0rdcycl0p.yml
```

---

# Contributor Schema

Every contributor file **must** contain all fields defined by the schema.

If a value is unknown, unavailable, or intentionally not disclosed, it **must** be set to `null`.

This guarantees a consistent structure across all contributor files and simplifies validation and automated processing.

Example:

```yaml
id: contributor-id
display_name: Display Name

real_name: null
email: null

profiles:
  github: username
  gitlab: null
  website: null

bio: null

roles:
  - contributor

areas:
  - documentation

joined: 2026

skills:
  languages: []
  concepts: []

location:
  timezone: null
  country: null

contact:
  preferred: github

agreements:
  cla: false
  dco: false

notes: null

markdown: null
```

unsure about the schema, look at this [CONTRIBUTOR file](./l0rdcycl0p.yml)

---

# Common Roles

Examples of common roles include:

* `maintainer`
* `developer`
* `contributor`
* `reviewer`
* `tester`
* `documentation`
* `translator`
* `designer`

Projects are free to define additional roles.

---

# Contribution Areas

The `areas` field describes the parts of the project a contributor primarily works on.

Examples include:

* `core`
* `parser`
* `cli`
* `api`
* `ui`
* `documentation`
* `testing`
* `ci`
* `build`

Projects may define additional areas as needed.

---

# Markdown Profile

Contributors may include a personal profile in the `markdown` field.

This content may be rendered on project websites, contributor pages, generated documentation, or other tools.

Every Markdown profile **must** begin with the following heading:

```markdown
# Hi, I'm {display_name} 👋
```

where `{display_name}` is the value of the `display_name` field.

Example:

```yaml
markdown: |
  # Hi, I'm L0rd Cycl0p 👋

  I'm a German software developer and the creator of this project.

  My interests include:
  - Programming languages
  - Compiler Design
  - Open Source
```

After the required heading, contributors are free to write anything they would like to share, including:

* A short introduction
* Technical expertise
* Interests and hobbies
* Favorite technologies
* Open-source experience
* Personal links
* Anything else appropriate for a public project

Please keep the content respectful, relevant, and suitable for all audiences.

---

# Guidelines

* Every contributor file **must** contain every field defined by the schema.
* Unknown or undisclosed values **must** be set to `null`.
* The filename **must** exactly match the value of `id`.
* The `id` **must** be unique within the project.
* Personal information such as a real name or email address is optional and may be `null`.
* Contributors should keep their information up to date.

