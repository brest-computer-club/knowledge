module Main exposing (main)

import Article
import Browser
import Html exposing (Html, div)
import Search


type alias Model =
    { search : Search.Model
    , article : Article.Model
    }


init : ( Model, Cmd Msg )
init =
    let
        ( sm, sc ) =
            Search.init

        ( am, ac ) =
            Article.init
    in
    ( { search = sm
      , article = am
      }
    , Cmd.batch [ Cmd.map SearchMsg sc, Cmd.map ArticleMsg ac ]
    )


type Msg
    = NoOp
    | SearchMsg Search.Msg
    | ArticleMsg Article.Msg


view : Model -> Html Msg
view m =
    div []
        [ Html.map SearchMsg <| Search.view m.search
        , Html.map ArticleMsg <| Article.view m.article
        ]


update : Msg -> Model -> ( Model, Cmd Msg )
update msg m =
    case msg of
        SearchMsg ms ->
            case ms of
                Search.GetArticle str ->
                    update (ArticleMsg (Article.GetArticle str)) m

                _ ->
                    let
                        ( sm, sc ) =
                            Search.update ms m.search
                    in
                    ( { m | search = sm }, Cmd.map SearchMsg sc )

        ArticleMsg ms ->
            let
                ( sm, sc ) =
                    Article.update ms m.article
            in
            ( { m | article = sm }, Cmd.map ArticleMsg sc )

        NoOp ->
            ( m, Cmd.none )


main : Program () Model Msg
main =
    Browser.application
        { init = \_ _ _ -> init
        , onUrlChange = \_ -> NoOp
        , onUrlRequest = \_ -> NoOp
        , subscriptions = \_ -> Sub.none
        , update = update
        , view = \m -> { title = "knowledge by the brest computer club", body = [ view m ] }
        }
