import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.github.thomasnield.rxkotlinfx.bind
import com.github.thomasnield.rxkotlinfx.observeOnFx
import io.reactivex.Flowable
import io.reactivex.Observable
import org.zeromq.ZMQ
import io.reactivex.rxkotlin.toObservable
import io.reactivex.subjects.PublishSubject
import javafx.application.Application
import javafx.scene.Parent
import tornadofx.*
import javafx.scene.layout.*
import javafx.scene.control.*
import javafx.application.*
import kotlinx.coroutines.experimental.Job
import kotlinx.coroutines.experimental.delay
import kotlinx.coroutines.experimental.newSingleThreadContext
import kotlinx.coroutines.experimental.reactive.consumeEach
import kotlinx.coroutines.experimental.reactive.openSubscription
import kotlinx.coroutines.experimental.reactive.publish

val ENDPOINT = "tcp://127.0.0.1:1234"

//////////////////////////////////////////
// Model
data class Entity(val name: String, val id: Long)

//////////////////////////////////////////
// Controller
class MainController: Controller() {
    val context : ZMQ.Context
    val socket: ZMQ.Socket
    val mapper = jacksonObjectMapper()
    val listenJobs = ArrayList<Job>()
    // We want to run ZMQ recv one only one thread
    val coroutineContext = newSingleThreadContext("ListenerContext")

    init {
        context = ZMQ.context(1)
        println("Connecting to server at $ENDPOINT")
        socket = context.socket(ZMQ.REQ)
        socket.connect(ENDPOINT)
        println("Connected")
    }

    inline fun <reified T: Any> watchCoroutine(id: String) = publish<T>(coroutineContext) {
        while (true) {
            delay(1000)
            socket.send(id)
            val json = socket.recvStr() // this may block, but not the UI thread
            val v = mapper.readValue<T>(json)
            println("Received value: $v")
            send(v)
        }
    }

    // This is executed by the UI thread
    inline fun <reified T: Any> remoteObservable(id: String) =
        Flowable.fromPublisher(watchCoroutine<T>(id)).observeOnFx()

    inline fun <reified T: Any> remoteWatch(id: String, crossinline action: (T) -> Unit) {
        // Create a flowable, then observe it on the UI thread
        remoteObservable<T>(id).subscribe {
            action(it)
        }
    }

    fun shutdown() {
        socket.close()
        context.term()
    }
}

//////////////////////////////////////////
// View
class MyView : View() {
    override val root = vbox{}
    val controller: MainController by inject()

    init {
        with(root) {
            vbox {
                textfield() {
                    textProperty().bind(controller.remoteObservable<Entity>("entity:0").map { it.toString() })
                }
                textfield() {
                    textProperty().bind(controller.remoteObservable<Entity>("entity:1").map { it.toString() })
                }
                textfield() {
                    textProperty().bind(controller.remoteObservable<Entity>("entity:2").map { it.toString() })
                }
            }
        }
    }
}

class MyApp : App(MyView::class)

fun main(args: Array<String>) {
    Application.launch(MyApp::class.java, *args)
}
