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


isDistantLink : String -> Bool
isDistantLink str =
    let
        reg =
            Maybe.withDefault Regex.never <|
                Regex.fromString "^http.*"
    in
    Regex.contains reg str


isLocalMarkdown : String -> Bool
isLocalMarkdown str =
    let
        reg =
            Maybe.withDefault Regex.never <|
                Regex.fromString ".*md$"
    in
    Regex.contains reg str


type DocType
    = Distant String
    | LocalAsset String
    | OtherArticle String


docType : String -> String -> DocType
docType path url =
    let
        normalizeLink link =
            normalizePath <| getFolder path ++ link
    in
    if isDistantLink url then
        Distant url

    else if isLocalMarkdown url then
        OtherArticle <|
            normalizeLink url

    else
        LocalAsset <|
            "/api/assets/"
                ++ toB64
                    (normalizeLink url)


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
                case docType path link.destination of
                    Distant url ->
                        Html.a
                            [ href url
                            , HA.target "_blank"
                            ]
                            content

                    LocalAsset url ->
                        Html.a
                            [ href url
                            , HA.target "_blank"
                            ]
                            content

                    OtherArticle url ->
                        Html.a
                            [ href "#"
                            , HE.onClick (GetArticle url)
                            ]
                            content
        , image =
            \imageInfo ->
                case Url.fromString imageInfo.src of
                    Just _ ->
                        Html.img
                            [ HA.style "max-width" "100%"
                            , HA.src imageInfo.src
                            ]
                            []

                    Nothing ->
                        let
                            imgPath =
                                "/api/assets/"
                                    ++ toB64
                                        (normalizeLink imageInfo.src)

                            isSVG =
                                Regex.contains (Maybe.withDefault Regex.never <| Regex.fromString "[a-z]+") imageInfo.src
                        in
                        if isSVG then
                            Html.object
                                [ HA.type_ "image/svg+xml"
                                , HA.attribute "data" imgPath
                                , HA.style "width" "100%"
                                , HA.style "min-height" "400px"
                                ]
                                []

                        else
                            Html.img
                                [ HA.style "max-width" "100%"
                                , HA.src imgPath
                                ]
                                []
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
