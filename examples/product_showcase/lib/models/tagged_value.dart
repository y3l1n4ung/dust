import 'package:derive_annotation/derive_annotation.dart';

import 'audit.dart';

part 'tagged_value.g.dart';

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
class TaggedValue with AuditStamp, _$TaggedValueDust {
  const TaggedValue({
    required this.code,
    required this.aliases,
  });

  final String code;
  final List<String> aliases;
}
