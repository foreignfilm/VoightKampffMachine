import Html exposing (Html, button, br, input, div, text)
import Html.Events exposing (onInput, onClick)
import WebSocket

type alias Model =
  { input : String
  , output : List String
  }
type Msg =
    Append(String)
  | Type(String)
  | Send

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    Append(s) ->
      ({ model | output = model.output ++ [s] }, Cmd.none)
    Type(s) ->
      ({ model | input = s }, Cmd.none)
    Send ->
      ({ model | input = "" }, WebSocket.send "ws://localhost:3030/echo" model.input)

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

subscriptions : Model -> Sub Msg
subscriptions model =
    WebSocket.listen "ws://localhost:3030/echo" Append

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
