import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

/// Issuing and recognising API tokens, on `package:cryptography`.
///
/// A token is a bearer credential: whoever holds it is the account. Three rules
/// follow, and this type exists so none of them is optional.
///
/// * **From a cryptographically secure source.** `SecretKeyData.random` rather
///   than `Random()`, and base64url so the bytes survive a header intact.
/// * **Stored as a hash, never in full.** A database read — a backup on a
///   laptop, a log line, an injection — then yields nothing usable.
/// * **Expiring.** A credential with no end date outlives the laptop it was
///   copied to.
abstract final class Tokens {
  /// How many random bytes a token carries. 32 is 256 bits.
  static const entropyBytes = 32;

  /// A new token, shown to the caller exactly once.
  static String issue() => base64Url
      .encode(SecretKeyData.random(length: entropyBytes).bytes)
      .replaceAll('=', '');

  /// What is stored and looked up.
  ///
  /// SHA-256 with no salt, deliberately: a token is already 256 bits of
  /// randomness, so there is nothing to guess and the lookup has to be by exact
  /// hash. Salting would make it unlookupable without making it stronger.
  static Future<String> fingerprint(String token) async {
    final digest = await Sha256().hash(utf8.encode(token));

    return _hex(Uint8List.fromList(digest.bytes));
  }

  static String _hex(Uint8List bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
