import Html exposing (Html, br, button, div, input, text)
import Html.Events exposing (onClick, onInput)
import Json.Decode exposing (Decoder, int, oneOf, string)
import Json.Decode.Pipeline exposing (decode, required)
import Json.Encode
import WebSocket


type alias SuspectId =
    String


type ServerCommand
    = Connected
    | BecomeSuspect { suspectId : SuspectId }
    | BecomeInvestigator { suspectId : SuspectId }
    | Echo { message : String }


type ClientCommand
    = LogInAsSuspect
    | LogInAsInvestigator { suspectId : SuspectId }
    | InvestigatorShout { message : String }


type Model
    = Nascent
    | Login
        { suspectId : SuspectId
        }
    | Suspect
        { suspectId : SuspectId
        , log : List String
        }
    | Investigator
        { suspectId : SuspectId
        , input : String
        }


type Msg
    = FromServer ServerCommand
    | ServerError
    | SuspectLogin
    | SetSuspectId String
    | InvestigatorLogin
    | Type String
    | Send


serverCommandFieldDecoder : String -> Decoder ServerCommand
serverCommandFieldDecoder t =
    case t of
        "Connected" ->
            Json.Decode.succeed Connected

        "BecomeSuspect" ->
            decode (\suspectId -> BecomeSuspect { suspectId = suspectId })
                |> required "suspect_id" string

        "BecomeInvestigator" ->
            decode (\suspectId -> BecomeInvestigator { suspectId = suspectId })
                |> required "suspect_id" string

        "Echo" ->
            decode (\message -> Echo { message = message })
                |> required "message" string

        _ ->
            Json.Decode.fail "Unexpected type"


serverCommandDecoder : Decoder ServerCommand
serverCommandDecoder =
    Json.Decode.field "type" string
        |> Json.Decode.andThen serverCommandFieldDecoder


encodeClientCommand : ClientCommand -> Json.Encode.Value
encodeClientCommand command =
    case command of
        LogInAsSuspect ->
            Json.Encode.object
                [ ( "type", Json.Encode.string "LogInAsSuspect" )
                ]

        LogInAsInvestigator logInAsInvestigator ->
            Json.Encode.object
                [ ( "type", Json.Encode.string "LogInAsInvestigator" )
                , ( "suspect_id", Json.Encode.string logInAsInvestigator.suspectId )
                ]

        InvestigatorShout shout ->
            Json.Encode.object
                [ ( "type", Json.Encode.string "InvestigatorShout" )
                , ( "message", Json.Encode.string shout.message )
                ]


sendClientCommand : ClientCommand -> Cmd Msg
sendClientCommand clientCommand =
    let
        json =
            encodeClientCommand clientCommand

        message =
            Json.Encode.encode 0 json
    in
    WebSocket.send "ws://localhost:3030/inhumanity" message


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        FromServer serverCommand ->
            case serverCommand of
                Connected ->
                    ( Login { suspectId = "" }, Cmd.none )

                BecomeSuspect becomeSuspect ->
                    ( Suspect
                        { suspectId = becomeSuspect.suspectId
                        , log = []
                        }
                    , Cmd.none
                    )

                BecomeInvestigator becomeInvestigator ->
                    ( Investigator
                        { suspectId = becomeInvestigator.suspectId
                        , input = ""
                        }
                    , Cmd.none
                    )

                Echo echo ->
                    case model of
                        Suspect suspect ->
                            ( Suspect
                                { suspect
                                    | log = suspect.log ++ [ echo.message ]
                                }
                            , Cmd.none
                            )

                        _ ->
                            ( model, Cmd.none )

        -- or what? show an error?
        ServerError ->
            ( model, Cmd.none )

        -- could consider displaying it on the client?
        SuspectLogin ->
            ( model, sendClientCommand LogInAsSuspect )

        SetSuspectId suspectId ->
            case model of
                Login login ->
                    ( Login { login | suspectId = suspectId }, Cmd.none )

                _ ->
                    ( model, Cmd.none )

        -- or what? show an error?
        InvestigatorLogin ->
            case model of
                Login login ->
                    ( model, sendClientCommand (LogInAsInvestigator { suspectId = login.suspectId }) )

                _ ->
                    ( model, Cmd.none )

        -- or what? show an error?
        Type s ->
            case model of
                Investigator investigator ->
                    ( Investigator
                        { investigator
                            | input = s
                        }
                    , Cmd.none
                    )

                _ ->
                    ( model, Cmd.none )

        -- or what? show an error?
        Send ->
            case model of
                Investigator investigator ->
                    ( Investigator { investigator | input = "" }, sendClientCommand (InvestigatorShout { message = investigator.input }) )

                _ ->
                    ( model, Cmd.none )



-- or what? show an error?


view : Model -> Html Msg
view model =
    case model of
        Nascent ->
            -- TODO: show a spinner
            div [] []

        Login login ->
            div []
                [ button [ onClick SuspectLogin ] [ text "Log in as suspect" ]
                , input [ onInput SetSuspectId ] [ text login.suspectId ]
                , button [ onClick InvestigatorLogin ] [ text "Log in as investigator" ]
                ]

        Suspect suspect ->
            div []
                [ Html.h2 [] [ text suspect.suspectId ]
                , div [] (List.intersperse (br [] []) (List.map text suspect.log))
                ]

        Investigator investigator ->
            div []
                [ Html.h2 [] [ text ("Investigating: " ++ investigator.suspectId) ]
                , input [ onInput Type ] [ text investigator.input ]
                , button [ onClick Send ] [ text "Shout" ]
                ]


parseWSMessage : String -> Msg
parseWSMessage s =
    Json.Decode.decodeString serverCommandDecoder s
        |> Result.map FromServer
        |> Result.withDefault ServerError


subscriptions : Model -> Sub Msg
subscriptions model =
    WebSocket.listen "ws://localhost:3030/inhumanity" parseWSMessage


main : Program Never Model Msg
main =
    Html.program
        { init = ( Nascent, Cmd.none )
        , update = update
        , view = view
        , subscriptions = subscriptions
        }
