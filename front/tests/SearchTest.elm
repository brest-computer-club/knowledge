module SearchTest exposing (..)

import Expect exposing (Expectation)
import Fuzz exposing (Fuzzer, int, list, string)
import Search exposing (..)
import Test exposing (..)


suite : Test
suite =
    let
        withTags tags =
            Input
                { id = "0"
                , op = Or
                , tags = tags
                , sub = []
                }

        withTagsAndSub ( tags, sub ) =
            Input
                { id = "0"
                , op = Or
                , tags = tags
                , sub = sub
                }
    in
    describe "Input to Query"
        [ describe "invalids return Nothing" <|
            [ test "empty tags" <|
                \_ ->
                    Expect.equal
                        Nothing
                        (inputToQuery (withTags []))
            , test "empty tags with subs" <|
                \_ ->
                    Expect.equal
                        Nothing
                        (inputToQuery
                            (withTagsAndSub ( [], [ withTags [ "a" ] ] ))
                        )
            ]
        , describe "without sub query" <|
            [ test "single tag return Sing" <|
                \_ ->
                    Expect.equal
                        (Just (Sing "a"))
                        (inputToQuery (withTags [ "a" ]))
            , test "2 tags " <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Sing "a")
                                (Sing "b")
                            )
                        )
                        (inputToQuery (withTags [ "a", "b" ]))
            , test "3 tags return Comb" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Sing "a")
                                (Comb Or
                                    (Sing "b")
                                    (Sing "c")
                                )
                            )
                        )
                        (inputToQuery (withTags [ "a", "b", "c" ]))
            , test "4 tags return Comb" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Sing "a")
                                (Comb Or
                                    (Sing "b")
                                    (Comb Or
                                        (Sing "c")
                                        (Sing "d")
                                    )
                                )
                            )
                        )
                        (inputToQuery (withTags [ "a", "b", "c", "d" ]))
            ]
        , describe "with sub query" <|
            [ test "1 tag, 1 sub" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Sing "a")
                                (Sing "b")
                            )
                        )
                        (inputToQuery (withTagsAndSub ( [ "a" ], [ withTags [ "b" ] ] )))
            , test "2 tags, 1 sub" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Comb Or
                                    (Sing "a")
                                    (Sing "b")
                                )
                                (Sing "c")
                            )
                        )
                        (inputToQuery (withTagsAndSub ( [ "a", "b" ], [ withTags [ "c" ] ] )))
            , test "2 tags, 2 sub" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Comb Or
                                    (Sing "a")
                                    (Sing "b")
                                )
                                (Comb Or
                                    (Sing "c")
                                    (Sing "d")
                                )
                            )
                        )
                        (inputToQuery
                            (withTagsAndSub
                                ( [ "a", "b" ]
                                , [ withTags [ "c" ], withTags [ "d" ] ]
                                )
                            )
                        )
            , test "2 tags, 2 sub, 1 nested" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Comb Or
                                    (Sing "a")
                                    (Sing "b")
                                )
                                (Comb Or
                                    (Comb Or (Sing "c") (Sing "d"))
                                    (Sing "e")
                                )
                            )
                        )
                        (inputToQuery
                            (withTagsAndSub
                                ( [ "a", "b" ]
                                , [ withTagsAndSub
                                        ( [ "c" ]
                                        , [ withTags [ "d" ] ]
                                        )
                                  , withTags [ "e" ]
                                  ]
                                )
                            )
                        )
            , test "2 tags, 3 sub" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Comb Or
                                    (Sing "a")
                                    (Sing "b")
                                )
                                (Comb Or
                                    (Sing "c")
                                    (Comb Or
                                        (Sing "d")
                                        (Sing "e")
                                    )
                                )
                            )
                        )
                        (inputToQuery
                            (withTagsAndSub
                                ( [ "a", "b" ]
                                , [ withTags [ "c" ]
                                  , withTags [ "d" ]
                                  , withTags [ "e" ]
                                  ]
                                )
                            )
                        )
            , test "2 tags, 3 sub nested" <|
                \_ ->
                    Expect.equal
                        (Just
                            (Comb Or
                                (Comb Or
                                    (Sing "a")
                                    (Sing "b")
                                )
                                (Comb Or
                                    (Sing "c")
                                    (Comb Or
                                        (Comb Or
                                            (Comb Or
                                                (Sing "d")
                                                (Sing "e")
                                            )
                                            (Sing "f")
                                        )
                                        (Comb Or
                                            (Sing "g")
                                            (Comb Or
                                                (Sing "h")
                                                (Sing "i")
                                            )
                                        )
                                    )
                                )
                            )
                        )
                        (inputToQuery
                            (withTagsAndSub
                                ( [ "a", "b" ]
                                , [ withTags [ "c" ]
                                  , withTagsAndSub ( [ "d", "e" ], [ withTags [ "f" ] ] )
                                  , withTags [ "g" ]
                                  , withTags [ "h" ]
                                  , withTags [ "i" ]
                                  ]
                                )
                            )
                        )
            ]
        ]
