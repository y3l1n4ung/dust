import 'dart:convert';
import 'dart:math';

import 'package:crypto/crypto.dart';

/// Issuing and recognising API tokens.
///
/// A token is a bearer credential: whoever holds it is the account. Three rules
/// follow, and this type exists so none of them is optional.
///
/// * **Generated from `Random.secure()`**, never a counter or a timestamp. A
///   guessable token is no token.
/// * **Stored as a hash**, never in full. A database read — a backup on a
///   laptop, a log line, an injection — then yields nothing usable.
/// * **Expiring.** A credential with no end date is one that outlives the
///   laptop it was copied to.
abstract final class Tokens {
  static final _random = Random.secure();

  /// A new token, shown to the caller exactly once.
  static String issue() {
    const alphabet =
        'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    return List.generate(
      40,
      (_) => alphabet[_random.nextInt(alphabet.length)],
    ).join();
  }

  /// What is stored and looked up.
  ///
  /// SHA-256 with no salt on purpose: a token is already 40 random characters,
  /// so it has nothing to guess and the lookup has to be by exact hash. Salting
  /// would make it unlookupable without also being any stronger.
  static String fingerprint(String token) =>
      sha256.convert(utf8.encode(token)).toString();
}
