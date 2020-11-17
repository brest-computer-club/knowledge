module Api exposing (..)

import Http
import Json.Decode as JD
import Json.Encode as JE


getTags : (Result Http.Error (List String) -> a) -> Cmd a
getTags m =
    Http.get
        { url = "/api/tags"
        , expect = Http.expectJson m (JD.list JD.string)
        }


type alias Article =
    { title : String
    , path : String
    }


articlesDecoder : JD.Decoder (List Article)
articlesDecoder =
    JD.list
        (JD.map2 Article
            (JD.field "title" JD.string)
            (JD.field "path" JD.string)
        )


getArticles : (Result Http.Error (List Article) -> a) -> Cmd a
getArticles m =
    Http.get
        { url = "/api/articles"
        , expect = Http.expectJson m articlesDecoder
        }


getArticlesByTag : (Result Http.Error (List Article) -> a) -> String -> Cmd a
getArticlesByTag m tag =
    Http.get
        { url = "/api/tags/" ++ tag
        , expect = Http.expectJson m articlesDecoder
        }


postSearchTags : JE.Value -> (Result Http.Error (List Article) -> a) -> Cmd a
postSearchTags jb m =
    Http.post
        { url = "/api/search-by-tags"
        , body = Http.jsonBody jb
        , expect = Http.expectJson m articlesDecoder
        }
