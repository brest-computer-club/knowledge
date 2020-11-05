module Main exposing (main)

import Browser
import Html exposing (Html, div, h1, text)
import Html.Events as HE


type alias Model =
    Int


init : Model
init =
    0


type Msg
    = Inc


view : Model -> Html Msg
view m =
    div []
        [ Html.button [ HE.onClick Inc ] [ text <| String.fromInt m ]
        ]


update : Msg -> Model -> Model
update _ m =
    m + 1


main : Program () Model Msg
main =
    Browser.sandbox
        { init = init
        , view = view
        , update = update
        }
