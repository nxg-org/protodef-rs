# starting point

we have a [`super::pds::ProtoDef`]
and want to get out a block of source code ([`proc_macro2::TokenStream`])

# detailed analysis of input

[`super::pds::ProtoDef`] is a HashMap of Paths mapping onto [`super::pds::Type`]s

Paths are built as follows:
"/namespace/namespace/typename"
the "types" property on the namespaces are removed

# what should output really look like?

Namespaces become modules

```rs
pub mod namespace {
   // types here
}
```

Types become structs if they can be represented as such

```rs
pub struct TypeName {
    // fields here
}
```

## native fields

```rs
pub struct TypeName {
    field_name: u64, // could be anything else
}
```

## struct fields

```rs
pub struct TypeName_FieldName{
    // subfields here
}

pub struct TypeName {
    field_name: TypeName_FieldName,
}
```

## enum fields

```rs
pub enum TypeName_FieldName{
    // cases here
}

pub struct TypeName {
    field_name: TypeName_FieldName,
}
```

### enum cases of native types (like u64) or aliases

```rs
pub enum TypeName_FieldName{
    U64(u64),
    I64(i64),
    // more cases...
}
```

### enum cases of structs

```rs
pub enum TypeName_FieldName{
    StructName{
        // fields here
    }
    // more cases...
}
```

### enum cases of enum cases

```rs
pub enum TypeName_FieldName{
    EnumCaseName(TypeName_FieldName_EnumCaseName),
    // more cases...
}

pub enum TypeName_FieldName_EnumCaseName{
    U64(u64),
    StructName{
        // fields here
    }
}
```

# how to get from one to the other

## getting something more usable from the pds::Types

how about a hashmap of code generating functions which you only 
have to pass the type name to and the detailed type it has so
you can construct structs and enums based on that?

## what to pass to a function that would get both of those from the pds::ProtoDef?

a `&pds::ProtoDef`, the `Path` to resolve and an `array of natives` i guess

## how to resolve then?

get the type at the path

match the type
- type is reference => look up reference then recurse
- type is container => do container stuff and recurse
- type is call => look up caller then call and return result
- type is native reference => 







for each entry in the ProtoDef (HashMap<Path, pds::Type>)


