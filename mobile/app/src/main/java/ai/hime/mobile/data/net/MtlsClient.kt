package ai.hime.mobile.data.net

import ai.hime.mobile.data.security.KeystoreManager
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong

class MtlsClient(
    private val keystoreManager: KeystoreManager
) {
    private val json = Json { ignoreUnknownKeys = true }
    private val reqCounter = AtomicLong(1)
    private var webSocket: WebSocket? = null

    private val _eventsFlow = MutableSharedFlow<String>(extraBufferCapacity = 64)
    val eventsFlow = _eventsFlow.asSharedFlow()

    private val okHttpClient = OkHttpClient.Builder()
        .connectTimeout(10, TimeUnit.SECONDS)
        .readTimeout(0, TimeUnit.MILLISECONDS) // Keep alive for WebSocket
        .build()

    fun connect(serverAddress: String, serverPort: Int) {
        val url = "ws://$serverAddress:$serverPort/ws"
        val request = Request.Builder()
            .url(url)
            .addHeader("X-Hime-Device-Fingerprint", keystoreManager.getClientCertFingerprint())
            .build()

        webSocket = okHttpClient.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                // Connection opened
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                _eventsFlow.tryEmit(text)
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                // Connection failure
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                // Disconnected
            }
        })
    }

    fun sendRpc(method: String, params: JsonObject): Long {
        val id = reqCounter.getAndIncrement()
        val rpcMsg = buildJsonObject {
            put("jsonrpc", "2.0")
            put("id", id)
            put("method", method)
            put("params", params)
        }
        webSocket?.send(rpcMsg.toString())
        return id
    }

    fun disconnect() {
        webSocket?.close(1000, "User initiated disconnect")
        webSocket = null
    }
}

