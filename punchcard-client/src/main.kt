import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
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

    init {
        context = ZMQ.context(1)
        println("Connecting to server at $ENDPOINT")
        socket = context.socket(ZMQ.REQ)
        socket.connect(ENDPOINT)
        println("Connected")
    }

    inline fun <reified T: Any> queryVariable(name: String): T {
        socket.send(name)
        val json = socket.recvStr()
        //println("Received json: $json")
        val v = mapper.readValue<T>(json)
        println("Received value: $v")
        return v
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
            textarea {
                text = controller.queryVariable<Entity>("entity:0").toString()
            }
        }
    }
}

class MyApp : App(MyView::class)


fun main(args: Array<String>) {
    Application.launch(MyApp::class.java, *args)


    // send a request for a specific entity

    // receive some entities
   /* for (i in 0..1000) {
    } */

    // disconnect
    //
}
