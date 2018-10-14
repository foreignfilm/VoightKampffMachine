import Html exposing (Html, button, br, input, div, text)
import Html.Events exposing (onInput, onClick)
import Json.Decode exposing (Decoder, int, string, oneOf)
import Json.Decode.Pipeline exposing (decode, required)
import Json.Encode
import WebSocket

type ServerCommand =
    Echo { message : String }
  | Timestamp { when : Int }

type ClientCommand =
    Shout { message : String }

type alias Model =
  { input : String
  , output : List String
  }
type Msg =
    FromServer(ServerCommand)
  | ServerError
  | Type(String)
  | Send

serverCommandFieldDecoder : String -> Decoder ServerCommand
serverCommandFieldDecoder t =
  case t of
    "Echo" ->
      decode (\message -> Echo { message = message })
        |> required "message" string
    "Timestamp" ->
      decode (\when -> Timestamp { when = when })
        |> required "when" int
    _ ->
      Json.Decode.fail "Unexpected type"

serverCommandDecoder : Decoder ServerCommand
serverCommandDecoder = 
  Json.Decode.field "type" string
    |> Json.Decode.andThen serverCommandFieldDecoder

encodeClientCommand : ClientCommand -> Json.Encode.Value
encodeClientCommand command =
  case command of
    Shout shout ->
      Json.Encode.object [
        ("type", Json.Encode.string "Shout"),
        ("message", Json.Encode.string shout.message)
      ]

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    FromServer(serverCommand) ->
      case serverCommand of
          Echo echo ->
            ({ model | output = model.output ++ [echo.message] }, Cmd.none)
          Timestamp timestamp ->
            ({ model | output = model.output ++ ["--" ++ (toString timestamp.when) ]}, Cmd.none)
    ServerError ->
       -- could consider displaying it on the client?
      (model, Cmd.none)
    Type(s) ->
      ({ model | input = s }, Cmd.none)
    Send ->
      let clientCommand = Shout { message = model.input }
          json = encodeClientCommand clientCommand
          message = Json.Encode.encode 0 json
      in
        ({ model | input = "" }, WebSocket.send "ws://localhost:3030/echo" message)

view : Model -> Html Msg
view model =
  div []
    [ div [] (List.intersperse (br [] []) (List.map text model.output))
    , input [onInput Type] [
        text model.input
      ]
    , button [onClick Send] [
        text "Send"
      ]
    ]

parseWSMessage : String -> Msg
parseWSMessage s =
  Json.Decode.decodeString serverCommandDecoder s
    |> Result.map FromServer
    |> Result.withDefault ServerError

subscriptions : Model -> Sub Msg
subscriptions model =
    WebSocket.listen "ws://localhost:3030/echo" parseWSMessage

main : Program Never Model Msg
main =
  Html.program {
    init = ({
      input = "",
      output = []
    }, Cmd.none),
    update = update,
    view = view,
    subscriptions = subscriptions
  }
