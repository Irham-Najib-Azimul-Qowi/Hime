package ai.hime.mobile.data.security

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyPair
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.MessageDigest
import java.security.Signature

class KeystoreManager {
    companion object {
        private const val ANDROID_KEYSTORE = "AndroidKeyStore"
        private const val CLIENT_KEY_ALIAS = "hime_device_client_identity"
    }

    private val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }

    fun getOrCreateDeviceKeyPair(): KeyPair {
        if (!keyStore.containsAlias(CLIENT_KEY_ALIAS)) {
            val keyPairGenerator = KeyPairGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_EC,
                ANDROID_KEYSTORE
            )
            val parameterSpec = KeyGenParameterSpec.Builder(
                CLIENT_KEY_ALIAS,
                KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY
            )
                .setDigests(KeyProperties.DIGEST_SHA256)
                .build()

            keyPairGenerator.initialize(parameterSpec)
            return keyPairGenerator.generateKeyPair()
        }

        val entry = keyStore.getEntry(CLIENT_KEY_ALIAS, null) as KeyStore.PrivateKeyEntry
        return KeyPair(entry.certificate.publicKey, entry.privateKey)
    }

    fun getClientCertFingerprint(): String {
        val keyPair = getOrCreateDeviceKeyPair()
        val encoded = keyPair.public.encoded
        val digest = MessageDigest.getInstance("SHA-256").digest(encoded)
        return "SHA256:" + digest.joinToString("") { "%02x".format(it) }
    }

    fun signPayload(payload: ByteArray): String {
        val entry = keyStore.getEntry(CLIENT_KEY_ALIAS, null) as KeyStore.PrivateKeyEntry
        val signature = Signature.getInstance("SHA256withECDSA").apply {
            initSign(entry.privateKey)
            update(payload)
        }
        val signedBytes = signature.sign()
        return signedBytes.joinToString("") { "%02x".format(it) }
    }
}
