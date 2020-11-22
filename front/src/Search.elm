port module Search exposing (..)

-- (Model, Msg(..), init, update, view)

import Api
import Html exposing (Html, button, div, li, text, ul)
import Html.Attributes as HA exposing (attribute, class)
import Html.Events as HE exposing (onClick)
import Http
import Json.Decode as JD exposing (field, list, string)
import Json.Encode as JE
import Regex


port notifyNewInput : ( String, List String ) -> Cmd msg


port notifyNewTags : List String -> Cmd msg


type alias Model =
    { inputs : Input
    , tags : List String
    , articles : List Article
    , open : Bool
    , filter : String
    }


type Input
    = Input
        { id : String
        , op : Op
        , tags : List String
        , sub : List Input
        }


type alias Path =
    String


type alias Article =
    { title : String
    , path : Path
    }


type Op
    = Or
    | And


type Query
    = Sing String
    | Comb Op Query Query


tagsToQuery : Op -> List String -> Maybe Query
tagsToQuery op l =
    case l of
        [] ->
            Nothing

        x1 :: x2 :: xs ->
            case tagsToQuery op xs of
                Just res ->
                    Just (Comb op (Sing x1) (Comb op (Sing x2) res))

                Nothing ->
                    Just (Comb op (Sing x1) (Sing x2))

        x :: _ ->
            Just (Sing x)


inputListToQuery : Op -> List Input -> Maybe Query
inputListToQuery op l =
    case l of
        [] ->
            Nothing

        x :: xs ->
            let
                ( mayXQ, mayXsHQ ) =
                    ( inputToQuery x
                    , inputListToQuery op xs
                    )
            in
            case ( mayXQ, mayXsHQ ) of
                ( Just xQ, Just xshQ ) ->
                    Just (Comb op xQ xshQ)

                ( Nothing, Just xshQ ) ->
                    Just xshQ

                ( Just xQ, Nothing ) ->
                    Just xQ

                ( Nothing, Nothing ) ->
                    Nothing


inputToQuery : Input -> Maybe Query
inputToQuery (Input i) =
    Maybe.andThen
        (\parent ->
            case inputListToQuery i.op i.sub of
                Nothing ->
                    Just parent

                Just subQ ->
                    Just (Comb i.op parent subQ)
        )
        (tagsToQuery i.op i.tags)


jsonOp : Op -> JE.Value
jsonOp op =
    case op of
        Or ->
            JE.string "or"

        And ->
            JE.string "and"


jsonQuery : Query -> JE.Value
jsonQuery q =
    case q of
        Sing str ->
            JE.object
                [ ( "sing"
                  , JE.object
                        [ ( "val", JE.string str ) ]
                  )
                ]

        Comb op qa qb ->
            JE.object
                [ ( "comb"
                  , JE.object
                        [ ( "op", jsonOp op )
                        , ( "qa", jsonQuery qa )
                        , ( "qb", jsonQuery qb )
                        ]
                  )
                ]


type Msg
    = InsertInputIn Input
    | GotTags (Result Http.Error (List String))
    | TagClicked String
    | GotArticles (Result Http.Error (List Article))
    | GetArticle String
    | Search Query
    | ToggleVisibility
    | InputChanged String InputModif
    | FilterUpdate String


type InputModif
    = InputTag String
    | InputOp Op


init : ( Model, Cmd Msg )
init =
    ( { tags = []
      , articles = []
      , inputs = inputFactory "root"
      , open = True
      , filter = ""
      }
    , Cmd.batch [ Api.getTags GotTags, Api.getArticles GotArticles, notifyNewInput ( "root", [] ) ]
    )


onChange : (String -> msg) -> Html.Attribute msg
onChange handler =
    HE.on "change" <|
        JD.map handler <|
            JD.at [ "target", "value" ] JD.string


inputDiv : Input -> Html Msg
inputDiv (Input i) =
    let
        inputOpToggle =
            case i.op of
                Or ->
                    div
                        [ HE.onClick <| InputChanged i.id (InputOp And)
                        , HA.style "cursor" "pointer"
                        ]
                        [ text "any" ]

                And ->
                    div
                        [ HE.onClick <| InputChanged i.id (InputOp Or)
                        , HA.style "cursor" "pointer"
                        ]
                        [ text "all" ]
    in
    div []
        [ inputOpToggle
        , div []
            [ button
                [ HA.style "float" "right"
                , HA.style "margin-top" "5px"
                , class "button button-outline button-small"
                , HE.onClick (InsertInputIn (Input i))
                ]
                [ text "+" ]
            ]
        , Html.input
            [ attribute "name" i.id
            , onChange (\str -> InputChanged i.id (InputTag str))
            , HA.attribute "value" <| String.join "," i.tags
            ]
            []
        , case i.sub of
            [] ->
                text ""

            subs ->
                div [ HA.style "padding-left" "20px" ] <|
                    List.map inputDiv subs
        ]


filterResults : String -> List Article -> List Article
filterResults str articles =
    if str == "" then
        articles

    else
        let
            reg =
                Maybe.withDefault Regex.never <|
                    Regex.fromString (".*" ++ str ++ ".*")
        in
        List.filter (\a -> Regex.contains reg a.title) articles


searchDiv : Model -> Html Msg
searchDiv m =
    let
        display =
            if not m.open then
                [ HA.style "display" "none" ]

            else
                [ HA.style "display" "block"
                ]
    in
    div display <|
        [ Html.h3 [] [ text "Knowledge" ]
        , div [] <|
            [ Html.h4 [] [ text "quick access" ]
            ]
                ++ (case m.tags of
                        [] ->
                            [ text "-- no tag found, please check the header of your markdown files --" ]

                        _ ->
                            List.map
                                (\t ->
                                    button
                                        [ HE.onClick (TagClicked t)
                                        , class "button"
                                        , class "button-small"
                                        ]
                                        [ text t ]
                                )
                            <|
                                List.sort m.tags
                   )
        , div []
            [ Html.hr [] []
            , Html.h4 [] [ text "advanced search" ]
            , inputDiv m.inputs
            , button
                [ HA.style "margin-top" "10px"
                , HA.style "float" "right"
                , case inputToQuery m.inputs of
                    Nothing ->
                        HA.disabled True

                    Just q ->
                        HE.onClick (Search q)
                ]
                [ text "search" ]
            ]
        , div
            [ HA.style "clear" "both"
            ]
            [ Html.hr [] []
            , Html.h4 [] [ text "results" ]
            , div []
                [ text "filter"
                , Html.input [ HA.value m.filter, HE.onInput FilterUpdate ] []
                ]
            , div
                [ HA.style "max-height" "300px"
                , HA.style "overflow-y" "auto"
                ]
                [ case m.articles of
                    [] ->
                        text "-- no result --"

                    _ ->
                        ul [] <|
                            List.map (\t -> li [ HE.onClick (GetArticle t.path), HA.style "cursor" "pointer" ] [ text t.title ]) <|
                                List.sortBy .title <|
                                    filterResults m.filter <|
                                        m.articles
                ]
            ]
        ]


view : Model -> Html Msg
view m =
    let
        visibilityButton : Bool -> Html Msg
        visibilityButton open =
            Html.button [ HA.style "float" "right", HA.class "button-outline", HE.onClick ToggleVisibility ]
                [ Html.text <|
                    if open then
                        "<<"

                    else
                        ">>"
                ]
    in
    div
        [ HA.style "height" "100vh"
        , HA.style "z-index" "100"
        , HA.style "position" "fixed"
        , HA.style "background" "white"
        , HA.style "border-right" "1px solid #e0e0e0"
        , HA.style "padding-top" "30px"
        , HA.style "max-width" "33%"
        , HA.style "overflow-y" "auto"
        , if m.open then
            HA.style "min-width" "33%"

          else
            HA.style "" ""
        ]
        [ div
            [ class "container" ]
            [ visibilityButton m.open
            , searchDiv m
            ]
        ]


updateInputByID : String -> (Input -> Input) -> Input -> Input
updateInputByID id f (Input input) =
    if input.id == id then
        f (Input input)

    else
        Input input


rangeInputs : (Input -> Input) -> Input -> Input
rangeInputs f (Input i) =
    let
        (Input tmp) =
            f (Input i)
    in
    Input { tmp | sub = List.map (rangeInputs f) tmp.sub }


appendChildInput : Input -> Input -> Input
appendChildInput child (Input inp) =
    Input { inp | sub = inp.sub ++ [ child ] }


setInputTags : List String -> Input -> Input
setInputTags tags (Input inp) =
    Input { inp | tags = tags }


setInputOp : Op -> Input -> Input
setInputOp op (Input inp) =
    Input { inp | op = op }


deserializeTags : String -> List String
deserializeTags str =
    let
        getValue =
            JD.field "value" JD.string
    in
    case JD.decodeString (JD.list getValue) str of
        Result.Ok res ->
            res

        Result.Err _ ->
            []


inputFactory : String -> Input
inputFactory id =
    Input
        { id = id
        , op = Or
        , tags = []
        , sub = []
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg m =
    case msg of
        FilterUpdate val ->
            ( { m | filter = val }, Cmd.none )

        Search q ->
            ( m, Api.postSearchTags (jsonQuery q) GotArticles )

        InputChanged id field ->
            case field of
                InputOp op ->
                    ( { m | inputs = rangeInputs (updateInputByID id (setInputOp op)) m.inputs }, Cmd.none )

                InputTag str ->
                    ( { m | inputs = rangeInputs (updateInputByID id (setInputTags (deserializeTags str))) m.inputs }, Cmd.none )

        ToggleVisibility ->
            ( { m | open = not m.open }, Cmd.none )

        InsertInputIn (Input i) ->
            let
                (Input child) =
                    inputFactory <| i.id ++ "-" ++ String.fromInt (List.length i.sub)

                newInputs =
                    rangeInputs (updateInputByID i.id (appendChildInput (Input child))) m.inputs
            in
            ( { m | inputs = newInputs }, notifyNewInput ( child.id, m.tags ) )

        GotTags res ->
            case res of
                Ok tags ->
                    ( { m | tags = tags }, notifyNewTags tags )

                Err _ ->
                    ( m, Cmd.none )

        TagClicked tag ->
            ( m, Cmd.batch [ Api.getTags GotTags, Api.getArticlesByTag GotArticles tag ] )

        GotArticles res ->
            case res of
                Ok articles ->
                    ( { m | articles = articles }, Cmd.none )

                Err _ ->
                    ( m, Cmd.none )

        GetArticle _ ->
            -- will be intercepted in main and routed to the article module
            ( m, Cmd.none )
