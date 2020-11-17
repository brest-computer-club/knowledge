module Article exposing (Model, Msg(..), init, update, view)

import Base64
import Bytes.Encode
import Html exposing (Html, div, text)
import Html.Attributes as HA exposing (class, href)
import Html.Events as HE exposing (onClick)
import Http exposing (get)
import Markdown.Parser as MDParser
import Markdown.Renderer as MDRenderer
import Regex
import Url


type alias Model =
    ( Path, String )


type alias Path =
    String


init : ( Model, Cmd Msg )
init =
    ( ( "", "" )
    , Cmd.none
    )


getArticle : String -> Cmd Msg
getArticle path =
    case
        Bytes.Encode.string path
            |> Bytes.Encode.encode
            |> Base64.fromBytes
    of
        Just artPath ->
            Http.get
                { url = "/api/articles/" ++ artPath
                , expect =
                    Http.expectString
                        (GotArticle path)
                }

        Nothing ->
            Cmd.none


type Msg
    = GetArticle String
    | GotArticle String (Result Http.Error String)


view : Model -> Html Msg
view m =
    div [ class "container", class "article", HA.style "padding-top" "30px" ]
        [ div [] <| removeYamlHeader True <| renderMarkdown m
        ]


removeYamlHeader : Bool -> List (Html a) -> List (Html a)
removeYamlHeader start src =
    case src of
        [] ->
            []

        x :: xs ->
            if start then
                removeYamlHeader False xs

            else if x == Html.hr [] [] then
                xs

            else
                removeYamlHeader False xs


renderMarkdown : ( Path, String ) -> List (Html Msg)
renderMarkdown ( path, str ) =
    case
        str
            |> MDParser.parse
            |> Result.mapError
                (\ee ->
                    String.join "\n" <|
                        List.map MDParser.deadEndToString ee
                )
            |> Result.andThen (\ast -> MDRenderer.render (customRenderer path) ast)
    of
        Ok rendered ->
            rendered

        Err errors ->
            [ text errors ]


customRenderer : String -> MDRenderer.Renderer (Html Msg)
customRenderer path =
    let
        orig =
            MDRenderer.defaultHtmlRenderer

        normalizeLink link =
            normalizePath <| getFolder path ++ link
    in
    { orig
        | link =
            \link content ->
                Html.a
                    [ href "#"
                    , HE.onClick (GetArticle <| normalizeLink link.destination)
                    ]
                    content
        , image =
            \imageInfo ->
                case Url.fromString imageInfo.src of
                    Just _ ->
                        Html.img [ HA.src imageInfo.src ] []

                    Nothing ->
                        Html.img [ HA.src <| "/api/images/" ++ toB64 (normalizeLink imageInfo.src) ] []
    }


getFolder : Path -> String
getFolder str =
    let
        reg =
            Maybe.withDefault Regex.never <|
                Regex.fromString ".*/"
    in
    str
        |> Regex.find reg
        |> List.map .match
        |> List.head
        |> Maybe.withDefault ""


normalizePath : Path -> String
normalizePath str =
    let
        reg =
            Maybe.withDefault Regex.never <|
                Regex.fromString "^(\\.\\.?/)*"

        prefix =
            str
                |> Regex.find reg
                |> List.map .match
                |> String.join ""
    in
    str
        |> String.split "/"
        |> List.foldl
            (\segment acc ->
                case segment of
                    ".." ->
                        List.drop 1 acc

                    "." ->
                        acc

                    _ ->
                        segment :: acc
            )
            []
        |> List.reverse
        |> (\l ->
                prefix
                    ++ String.join "/" l
           )


toB64 : String -> String
toB64 str =
    case
        Bytes.Encode.string str
            |> Bytes.Encode.encode
            |> Base64.fromBytes
    of
        Just b ->
            b

        Nothing ->
            ""


update : Msg -> Model -> ( Model, Cmd Msg )
update msg m =
    case msg of
        GetArticle path ->
            ( m, getArticle path )

        GotArticle path res ->
            case res of
                Ok content ->
                    ( ( path, content ), Cmd.none )

                Err _ ->
                    ( m, Cmd.none )
