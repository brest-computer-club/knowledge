module Main exposing (main)

import Base64
import Browser
import Bytes.Encode
import Html exposing (Html, button, div, h1, li, text, ul)
import Html.Events as HE exposing (onClick)
import Http exposing (expectJson, get)
import Json.Decode exposing (Decoder, field, list, map2, string)
import Markdown.Parser as Markdown
import Markdown.Renderer


type alias Model =
    { tags : List String
    , articles : List Article
    , article : String
    }


type alias Article =
    { title : String
    , path : String
    }


init : ( Model, Cmd Msg )
init =
    ( { tags = [], articles = [], article = "" }
    , Http.get
        { url = "http://127.0.0.1:8080/tags"
        , expect = Http.expectJson GotTags tagDecoder
        }
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
                { url = "http://127.0.0.1:8080/article/" ++ artPath
                , expect = Http.expectString GotArticle
                }

        Nothing ->
            Cmd.none


getArticlesByTag : String -> Cmd Msg
getArticlesByTag tag =
    Http.get
        { url = "http://127.0.0.1:8080/tag/" ++ tag
        , expect = Http.expectJson GotArticlesByTag articlesDecoder
        }


articlesDecoder : Decoder (List Article)
articlesDecoder =
    list
        (map2 Article
            (field "title" string)
            (field "path" string)
        )


tagDecoder : Decoder (List String)
tagDecoder =
    list string


type Msg
    = NoOp
    | GotTags (Result Http.Error (List String))
    | GetArticlesByTag String
    | GotArticlesByTag (Result Http.Error (List Article))
    | GetArticle String
    | GotArticle (Result Http.Error String)


view : Model -> Html Msg
view m =
    div [] <|
        [ h1 [] [ text "knowledge" ]
        , div [] (List.map (\t -> button [ HE.onClick (GetArticlesByTag t) ] [ text t ]) m.tags)
        , div [] [ ul [] <| List.map (\t -> li [ HE.onClick (GetArticle t.path) ] [ text t.title ]) m.articles ]
        , div [] [ renderMarkdown m.article ]
        ]


renderMarkdown : String -> Html msg
renderMarkdown str =
    case
        str
            |> Markdown.parse
            |> Result.mapError
                (\ee ->
                    String.join "\n" <|
                        List.map Markdown.deadEndToString ee
                )
            |> Result.andThen (\ast -> Markdown.Renderer.render Markdown.Renderer.defaultHtmlRenderer ast)
    of
        Ok rendered ->
            div [] rendered

        Err errors ->
            text errors


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

        GetArticlesByTag tag ->
            ( m, getArticlesByTag tag )

        GotArticlesByTag res ->
            case res of
                Ok articles ->
                    ( { m | articles = articles }, Cmd.none )

                Err _ ->
                    ( m, Cmd.none )

        GetArticle path ->
            ( m, getArticle path )

        GotArticle res ->
            case res of
                Ok content ->
                    ( { m | article = content }, Cmd.none )

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
