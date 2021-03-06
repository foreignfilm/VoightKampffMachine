module Main exposing (main)

import Html exposing (Html, br, button, div, input, text)
import Html.Events exposing (onClick, onInput)
import Json.Decode
import Json.Encode
import WebSocket

import Commands exposing (SuspectId, ServerCommand(..), ClientCommand(..), encodeClientCommand, serverCommandDecoder)


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
                            -- or what? show an error?
                            ( model, Cmd.none )

        ServerError ->
            -- could consider displaying it on the client?
            ( model, Cmd.none )

        SuspectLogin ->
            ( model, sendClientCommand LogInAsSuspect )

        SetSuspectId suspectId ->
            case model of
                Login login ->
                    ( Login { login | suspectId = suspectId }, Cmd.none )

                _ ->
                    -- or what? show an error?
                    ( model, Cmd.none )

        InvestigatorLogin ->
            case model of
                Login login ->
                    ( model, sendClientCommand (LogInAsInvestigator { suspectId = login.suspectId }) )

                _ ->
                    -- or what? show an error?
                    ( model, Cmd.none )

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
                    -- or what? show an error?
                    ( model, Cmd.none )

        Send ->
            case model of
                Investigator investigator ->
                    ( Investigator { investigator | input = "" }, sendClientCommand (InvestigatorShout { message = investigator.input }) )

                _ ->
                    -- or what? show an error?
                    ( model, Cmd.none )


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
