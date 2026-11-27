package com.velmora.stepbyte.velmora.account

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.io.File
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * A small secret kept encrypted with a key that never leaves the Android
 * Keystore (AES-256-GCM). Used for the account's refresh token, the Android
 * side of desktop's Credential Manager. The file is outside auto-backup, so
 * the token never travels to another phone.
 */
internal class SecureStore(context: Context, name: String) {
  private val file = File(context.applicationContext.noBackupFilesDir, "$name.bin")
  private val alias = "velmora.$name"

  private fun key(): SecretKey {
    val store = KeyStore.getInstance(KEYSTORE).apply { load(null) }
    (store.getKey(alias, null) as? SecretKey)?.let {
      return it
    }
    val gen = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE)
    gen.init(
      KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setKeySize(256)
        .build(),
    )
    return gen.generateKey()
  }

  fun write(text: String) {
    val cipher = Cipher.getInstance(TRANSFORM)
    cipher.init(Cipher.ENCRYPT_MODE, key())
    val iv = cipher.iv
    val sealed = cipher.doFinal(text.toByteArray())
    val tmp = File(file.path + ".tmp")
    tmp.writeBytes(byteArrayOf(iv.size.toByte()) + iv + sealed)
    tmp.renameTo(file)
  }

  /** The secret, or null if there is none or it can't be opened (a new Keystore key). */
  fun read(): String? =
    runCatching {
        val bytes = file.readBytes()
        val n = bytes[0].toInt()
        val iv = bytes.copyOfRange(1, 1 + n)
        val cipher = Cipher.getInstance(TRANSFORM)
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, iv))
        String(cipher.doFinal(bytes.copyOfRange(1 + n, bytes.size)))
      }
      .getOrNull()

  fun clear() {
    file.delete()
  }

  private companion object {
    const val KEYSTORE = "AndroidKeyStore"
    const val TRANSFORM = "AES/GCM/NoPadding"
  }
}
