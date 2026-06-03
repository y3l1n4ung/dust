import 'package:dust_dart/derive.dart';

import 'audit.dart';

part 'tagged_value.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class TaggedValue with AuditStamp, _$TaggedValue {
  const TaggedValue({required this.code, required this.aliases});

  final String code;
  final List<String> aliases;
}
