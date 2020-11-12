module Main exposing (main)

import Base64
import Browser
import Bytes.Encode
import Html exposing (Html, button, div, h1, li, text, ul)
import Html.Attributes exposing (class, href)
import Html.Events as HE exposing (onClick)
import Http exposing (expectJson, get)
import Json.Decode exposing (Decoder, field, list, map2, string)
import Markdown.Parser as MDParser
import Markdown.Renderer as MDRenderer
import Url


type alias Model =
    { tags : List String
    , articles : List Article
    , articleContent : ( Path, String )
    }


type alias Article =
    { title : String
    , path : Path
    }


type alias Path =
    String


init : ( Model, Cmd Msg )
init =
    ( { tags = [], articles = [], articleContent = ( "", "" ) }
    , getTags
    )


getTags : Cmd Msg
getTags =
    Http.get
        { url = "/api/tags"
        , expect = Http.expectJson GotTags (list string)
        }


getArticle : String -> Cmd Msg
getArticle path =
    case
        Bytes.Encode.string path
            |> Bytes.Encode.encode
            |> Base64.fromBytes
    of
        Just artPath ->
            Http.get
                { url = "/api/article/" ++ artPath
                , expect =
                    Http.expectString
                        (GotArticle path)
                }

        Nothing ->
            Cmd.none


getArticlesByTag : String -> Cmd Msg
getArticlesByTag tag =
    Http.get
        { url = "/api/tag/" ++ tag
        , expect = Http.expectJson GotArticlesByTag articlesDecoder
        }


articlesDecoder : Decoder (List Article)
articlesDecoder =
    list
        (map2 Article
            (field "title" string)
            (field "path" string)
        )


type Msg
    = NoOp
    | GotTags (Result Http.Error (List String))
    | TagClicked String
    | GotArticlesByTag (Result Http.Error (List Article))
    | GetArticle String
    | GotArticle String (Result Http.Error String)


view : Model -> Html Msg
view m =
    div [ class "container" ] <|
        [ h1 [] [ text "" ]
        , div [] (List.map (\t -> button [ HE.onClick (TagClicked t), class "button", class "button-small" ] [ text t ]) m.tags)
        , div [] [ ul [] <| List.map (\t -> li [ HE.onClick (GetArticle t.path) ] [ text t.title ]) m.articles ]
        , div [] <| removeYamlHeader True <| renderMarkdown m.articleContent
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
            stringNormalize <| getFolder path ++ link
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
                        Html.img [ Html.Attributes.src imageInfo.src ] []

                    Nothing ->
                        Html.img [ Html.Attributes.src <| "/api/images/" ++ (toB64 <| normalizeLink imageInfo.src) ] []
    }


getFolder : String -> String
getFolder str =
    str
        |> String.split "/"
        |> List.take -1
        |> String.join "/"


stringNormalize : String -> String
stringNormalize str =
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
        |> String.join "/"


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
        NoOp ->
            ( m, Cmd.none )

        GotTags res ->
            case res of
                Ok tags ->
                    ( { m | tags = tags }, Cmd.none )

                Err _ ->
                    ( m, Cmd.none )

        TagClicked tag ->
            ( m, Cmd.batch [ getTags, getArticlesByTag tag ] )

        GotArticlesByTag res ->
            case res of
                Ok articles ->
                    ( { m | articles = articles }, Cmd.none )

                Err _ ->
                    ( m, Cmd.none )

        GetArticle path ->
            ( m, getArticle path )

        GotArticle path res ->
            case res of
                Ok content ->
                    ( { m | articleContent = ( path, content ) }, Cmd.none )

                Err _ ->
                    ( m, Cmd.none )


main : Program () Model Msg
main =
    Browser.application
        { init = \_ _ _ -> init
        , onUrlChange = \_ -> NoOp
        , onUrlRequest = \_ -> NoOp
        , subscriptions = \_ -> Sub.none
        , update = update
        , view = \m -> { title = "knowledge", body = [ view m ] }
        }
