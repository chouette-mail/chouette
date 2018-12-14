module Main exposing (main)

import Browser
import Element exposing (Element, alignRight, alignTop, centerX, centerY, el, fill, padding, rgb255, row, spacing, text, width)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Http
import Json.Encode


main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }



-- TYPES ----------------------------------------------------------------------


type alias Account =
    { username : String
    , email : String
    , password : String
    }


emptyAccount : Account
emptyAccount =
    { username = "", email = "", password = "" }


accountToJson : Account -> Json.Encode.Value
accountToJson account =
    Json.Encode.object
        [ ( "username", Json.Encode.string account.username )
        , ( "email", Json.Encode.string account.email )
        , ( "password", Json.Encode.string account.password )
        ]


type NewAccountFormMessage
    = UsernameChanged String
    | EmailChanged String
    | PasswordChanged String
    | NewAccountSubmitted


type Msg
    = NewAccountClicked
    | NewAccountFormMessage NewAccountFormMessage
    | NewAccountRegistered (Result Http.Error String)


type Model
    = LogIn
    | Subscribing Account
    | Home Account
    | NewAccount Account


init : () -> ( Model, Cmd Msg )
init _ =
    ( LogIn, Cmd.none )



-- UPDATES --------------------------------------------------------------------


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NewAccountClicked ->
            ( NewAccount emptyAccount, Cmd.none )

        NewAccountFormMessage formMsg ->
            updateNewAccountForm formMsg model

        NewAccountRegistered content ->
            ( model, Cmd.none )


updateNewAccountForm : NewAccountFormMessage -> Model -> ( Model, Cmd Msg )
updateNewAccountForm msg model =
    case ( msg, model ) of
        ( UsernameChanged newUsername, NewAccount currentAccount ) ->
            ( NewAccount { currentAccount | username = newUsername }, Cmd.none )

        ( EmailChanged newEmail, NewAccount currentAccount ) ->
            ( NewAccount { currentAccount | email = newEmail }, Cmd.none )

        ( PasswordChanged newPassword, NewAccount currentAccount ) ->
            ( NewAccount { currentAccount | password = newPassword }, Cmd.none )

        ( NewAccountSubmitted, NewAccount currentAccount ) ->
            ( Subscribing currentAccount
            , Http.post
                { url = "/new-user"
                , body = Http.jsonBody (accountToJson currentAccount)
                , expect = Http.expectString NewAccountRegistered
                }
            )

        _ ->
            ( model, Cmd.none )



-- SUBSCRIPTIONS --------------------------------------------------------------


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- VIEWS ----------------------------------------------------------------------


defaultAttributes : List (Element.Attribute msg)
defaultAttributes =
    [ padding 5, spacing 5 ]


view model =
    case model of
        LogIn ->
            logInView

        Home account ->
            homeView account

        NewAccount account ->
            newAccountView account

        Subscribing account ->
            subscribingView


logInView =
    Element.layout defaultAttributes
        myRowOfStuff


homeView account =
    Element.layout defaultAttributes
        Element.none


subscribingView =
    Element.layout defaultAttributes
        Element.none


newAccountView account =
    Element.layout defaultAttributes
        (Element.column (centerX :: defaultAttributes)
            [ Input.text defaultAttributes
                { onChange = NewAccountFormMessage << UsernameChanged
                , label = Input.labelAbove (centerY :: defaultAttributes) (text "Username")
                , placeholder = Nothing
                , text = account.username
                }
            , Input.email defaultAttributes
                { onChange = NewAccountFormMessage << EmailChanged
                , label = Input.labelAbove (centerY :: defaultAttributes) (text "Email address")
                , placeholder = Nothing
                , text = account.email
                }
            , Input.newPassword defaultAttributes
                { onChange = NewAccountFormMessage << PasswordChanged
                , label = Input.labelAbove (centerY :: defaultAttributes) (text "Password")
                , placeholder = Nothing
                , text = account.password
                , show = False
                }
            , Input.button
                (Border.rounded 5 :: Border.width 1 :: centerX :: defaultAttributes)
                { onPress = Just (NewAccountFormMessage NewAccountSubmitted)
                , label = text "New account"
                }
            ]
        )


addAccount : Element Msg
addAccount =
    Input.button
        (Border.rounded 5 :: Border.width 1 :: defaultAttributes)
        { onPress = Just NewAccountClicked
        , label = text "New account"
        }


myRowOfStuff : Element Msg
myRowOfStuff =
    row [ width fill, alignTop, spacing 30 ]
        [ el (alignRight :: defaultAttributes) addAccount
        ]
