# Diff

Diff support is available natively in zed-custom.

- Tree-sitter: [zed-industries/the-mikedavis/tree-sitter-diff](https://github.com/the-mikedavis/tree-sitter-diff)

## Configuration

zed-custom will not attempt to format diff files and has [`remove_trailing_whitespace_on_save`](https://zed.dev/docs/reference/all-settings#remove-trailing-whitespace-on-save) and [`ensure-final-newline-on-save`](https://zed.dev/docs/reference/all-settings#ensure-final-newline-on-save) set to false.

zed-custom will automatically recognize files with `patch` and `diff` extensions as Diff files. To recognize other extensions, add them to `file_types` in your zed-custom settings.json:

```json [settings]
  "file_types": {
    "Diff": ["dif"]
  },
```
