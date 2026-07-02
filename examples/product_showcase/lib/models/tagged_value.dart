import 'package:dust_dart/derive.dart';

import 'audit.dart';

part 'tagged_value.g.dart';

/// Tagged value model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class TaggedValue with AuditStamp, _$TaggedValue {
  /// Creates a [TaggedValue].
  const TaggedValue({required this.code, required this.aliases});

  /// Code.
  final String code;

  /// Aliases.
  final List<String> aliases;
}
