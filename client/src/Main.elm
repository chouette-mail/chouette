module Main exposing (main)

import Browser
import Element exposing (Element, alignRight, alignTop, centerX, centerY, el, fill, padding, rgb255, row, spacing, text, width)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input


main =
    Browser.sandbox { init = initialScreen, update = update, view = view }



-- TYPES ----------------------------------------------------------------------


type alias Account =
    { username : String
    , email : String
    , password : String
    }


emptyAccount : Account
emptyAccount =
    { username = "", email = "", password = "" }


type NewAccountFormMessage
    = UsernameChanged String
    | EmailChanged String
    | PasswordChanged String
    | NewAccountSubmitted


type Msg
    = NewAccountClicked
    | NewAccountFormMessage NewAccountFormMessage


type Model
    = LogIn
    | Home Account
    | NewAccount Account



-- UPDATES --------------------------------------------------------------------


update : Msg -> Model -> Model
update msg model =
    case msg of
        NewAccountClicked ->
            NewAccount emptyAccount

        NewAccountFormMessage formMsg ->
            updateNewAccountForm formMsg model


updateNewAccountForm : NewAccountFormMessage -> Model -> Model
updateNewAccountForm msg model =
    case ( msg, model ) of
        ( UsernameChanged newUsername, NewAccount currentAccount ) ->
            NewAccount { currentAccount | username = newUsername }

        ( EmailChanged newEmail, NewAccount currentAccount ) ->
            NewAccount { currentAccount | email = newEmail }

        ( PasswordChanged newPassword, NewAccount currentAccount ) ->
            NewAccount { currentAccount | password = newPassword }

        ( NewAccountSubmitted, NewAccount currentAccount ) ->
            Home currentAccount

        _ ->
            model



-- VIEWS ----------------------------------------------------------------------


defaultAttributes : List (Element.Attribute msg)
defaultAttributes =
    [ padding 5, spacing 5 ]


initialScreen : Model
initialScreen =
    LogIn


view model =
    case model of
        LogIn ->
            logInView

        Home account ->
            homeView account

        NewAccount account ->
            newAccountView account


logInView =
    Element.layout defaultAttributes
        myRowOfStuff


homeView account =
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
