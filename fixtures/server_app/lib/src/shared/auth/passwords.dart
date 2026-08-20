import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';
import 'package:cryptography/helpers.dart';

/// Hashing and verifying passwords, on `package:cryptography`.
///
/// **Argon2id**, which is what OWASP recommends first for password storage and
/// what the Password Hashing Competition selected. It is memory-hard, so the
/// GPU and ASIC arrays that make PBKDF2 and bcrypt cheap to attack have to buy
/// RAM per guess as well as compute.
///
/// The parameters below are OWASP's minimum configuration: 19 MiB of memory,
/// two iterations, one degree of parallelism. They are the security setting, so
/// they are named constants rather than numbers buried in a call — raise
/// [memoryKiB] until hashing costs what you can afford, and treat any change as
/// something that has to be recorded per hash before you make it.
///
/// The library does the parts that are easy to get wrong: the KDF itself, and a
/// constant-time comparison. Nothing here is hand-rolled.
abstract final class Passwords {
  /// Memory cost, in kibibytes. OWASP's floor is 19 MiB.
  static const int memoryKiB = 19 * 1024;

  /// How many passes over memory.
  static const iterations = 2;

  /// How many lanes may run at once.
  static const parallelism = 1;

  static const _hashLength = 32;
  static const _saltLength = 16;

  static final Argon2id _algorithm = Argon2id(
    memory: memoryKiB,
    iterations: iterations,
    parallelism: parallelism,
    hashLength: _hashLength,
  );

  /// A new random salt, hex encoded.
  ///
  /// From the library's own secure random rather than `Random.secure()`, so
  /// there is one source of randomness to reason about.
  static String newSalt() => _hex(
        Uint8List.fromList(SecretKeyData.random(length: _saltLength).bytes),
      );

  /// Derives the hash of [password] with [salt], hex encoded.
  static Future<String> hash(String password, String salt) async {
    final derived = await _algorithm.deriveKey(
      secretKey: SecretKey(utf8.encode(password)),
      nonce: utf8.encode(salt),
    );

    return _hex(Uint8List.fromList(await derived.extractBytes()));
  }

  /// Whether [password] matches [expectedHash].
  ///
  /// Compared with the library's constant-time equality. A `==` on a secret
  /// returns as soon as two bytes differ, and how long that took tells an
  /// attacker how much of their guess was right — enough to recover a hash one
  /// byte at a time given enough requests.
  static Future<bool> verify(
    String password,
    String salt,
    String expectedHash,
  ) async {
    final actual = await hash(password, salt);

    return constantTimeBytesEquality.equals(
      utf8.encode(actual),
      utf8.encode(expectedHash),
    );
  }

  static String _hex(Uint8List bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
