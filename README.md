# foolang

## Specificationish

Smalltalk syntax, except:

- Blocks use {}
- Blocks have implicit argument _.
- x => { ... } desugars into { ... } value: x

## Motivating Examples

### Example 1

Using blocks:

    Backend select: #postgres => {
      _ connect: "localhost" => {
          _ query "select * from users" do: { "User: {_ name}" print }.
          _ query "select * from suppliers" do: { "Vendor: {_ name}" print }.
      }.
      _ connect: "remote" => {
        _ query "select * from users" do: { "User: {_ name}" print }.
        _ query "select * from suppliers" do: { "Vendor: {_ name}" print }.
      }.
    }.

Factored using phrases:

    define -print-names-in-table: table as: pretty {
      _ query "select * from {table}" do: { "{pretty}: {_ name}" print }
    }

    define -print-main-tables {
      _ -print-names-in-table "users" as: "User";
      _ -print-names-in-table "suppliers" as: "Vendor"
    }

    Backend select: #postgres => {
      _ connect: "localhost" -print-main-tables.
      _ connect: "remote" -print-main-tables.
    }.
