import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

/// Hashing and verifying passwords.
///
/// PBKDF2-HMAC-SHA256, which `package:crypto` can build from `Hmac`. It is the
/// weakest of the acceptable choices — Argon2id or scrypt resist GPUs better —
/// and it is what a pure-Dart application can reach without a native
/// dependency. The iteration count is the knob: raise it until hashing costs
/// about 100ms on your hardware, and store it if you ever want to change it
/// without invalidating every password.
///
/// What matters as much as the algorithm:
///
/// * a **per-account salt**, so one rainbow table cannot cover two accounts and
///   two users with the same password do not have the same hash;
/// * a **constant-time** comparison, because a byte-at-a-time comparison leaks
///   the answer through timing;
/// * never storing the password, and never logging it.
abstract final class Passwords {
  /// How many rounds. Deliberately a named constant: it is the security
  /// parameter, and a number buried in a function is one nobody revisits.
  static const iterations = 120000;

  static const _keyLength = 32;
  static final _random = Random.secure();

  /// A new random salt, hex encoded.
  static String newSalt() {
    final bytes = Uint8List.fromList(
      List.generate(16, (_) => _random.nextInt(256)),
    );
    return _hex(bytes);
  }

  /// Derives the hash of [password] with [salt], hex encoded.
  static String hash(String password, String salt) {
    final hmac = Hmac(sha256, utf8.encode(password));
    final saltBytes = utf8.encode(salt);

    // PBKDF2: the first block is enough for a 32-byte key with SHA-256.
    var block = Uint8List.fromList(
      hmac.convert([...saltBytes, 0, 0, 0, 1]).bytes,
    );
    final result = Uint8List.fromList(block);

    for (var round = 1; round < iterations; round++) {
      block = Uint8List.fromList(hmac.convert(block).bytes);
      for (var index = 0; index < result.length; index++) {
        result[index] ^= block[index];
      }
    }

    return _hex(Uint8List.sublistView(result, 0, _keyLength));
  }

  /// Whether [password] matches [expectedHash], compared in constant time.
  static bool verify(String password, String salt, String expectedHash) =>
      constantTimeEquals(hash(password, salt), expectedHash);

  /// Compares without stopping at the first difference.
  ///
  /// `==` returns as soon as two bytes differ, and the time that takes tells an
  /// attacker how much of their guess was right — enough to recover a secret
  /// one byte at a time over enough requests.
  static bool constantTimeEquals(String a, String b) {
    if (a.length != b.length) return false;

    var difference = 0;
    for (var index = 0; index < a.length; index++) {
      difference |= a.codeUnitAt(index) ^ b.codeUnitAt(index);
    }
    return difference == 0;
  }

  static String _hex(Uint8List bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
