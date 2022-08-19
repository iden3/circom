# Pragma

## Version pragma

All files with .circom extension should start with a first `pragma` instruction specifying the compiler version, like this: 

```text
pragma circom xx.yy.zz;
```

This is to ensure that the circuit is compatible with the compiler version indicated after the `pragma` instruction. Otherwise, the compiler throws a warning.

If a file does not contain this instruction, it is assumed that the code is compatible with the latest compiler's version and a warning is thrown.

## Custom templates pragma

Since circom 2.0.6, the language allows the definition of custom templates (see [this](../circom-language/templates-and-components.md#custom-templates) for more information). This `pragma` allows the circom programmer to easily tell if it's using custom templates: if any file declaring a custom template or including a file declaring any custom template doesn't use this `pragma`, the compiler will produce an error. Moreover, it will inform the programmer about which files should include this pragma.

To use it simply add the following instruction at the beginning (and after the version `pragma`) of the .circom files that needs it:

```text
pragma custom_templates;
```
