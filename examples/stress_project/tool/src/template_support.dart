String mixinName(String className) => '_\$${className}Dust';

String serdeFactory(String className) =>
    "  factory $className.fromJson(Map<String, Object?> json) => _\$${className}FromJson(json);";

String renderFile({
  required String fileName,
  required List<String> imports,
  required List<String> declarations,
}) => [
  ...imports,
  '',
  "part '$fileName.g.dart';",
  '',
  ...declarations,
  '',
].join('\n');
