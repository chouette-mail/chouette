module Main exposing (main)

import Browser
import Element exposing (Element, alignRight, alignTop, centerX, centerY, column, el, fill, fillPortion, height, link, padding, paddingXY, paragraph, rgb255, row, spacing, text, width)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Http
import Styles exposing (colors)


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


type alias ImapAccount =
    { imapServer : String
    , username : String
    , password : String
    }


type alias LogInFormContent =
    { username : String
    , password : String
    }


emptyAccount : Account
emptyAccount =
    { username = "", email = "", password = "" }


emptyLogInFormContent : LogInFormContent
emptyLogInFormContent =
    { username = "", password = "" }


emptyImapAccount : ImapAccount
emptyImapAccount =
    { username = "", imapServer = "", password = "" }


accountToUrlEncoded : Account -> String
accountToUrlEncoded account =
    String.join "&"
        [ "username=" ++ account.username
        , "email=" ++ account.email
        , "password=" ++ account.password
        ]


logInFormContentToUrlEncoded : LogInFormContent -> String
logInFormContentToUrlEncoded account =
    String.join "&"
        [ "username=" ++ account.username
        , "password=" ++ account.password
        ]


addImapServerFormContentToUrlEncoded : ImapAccount -> String
addImapServerFormContentToUrlEncoded account =
    String.join "&"
        [ "username=" ++ account.username
        , "password=" ++ account.password
        , "server=" ++ account.imapServer
        ]


type AddImapAccountFormMessage
    = AddImapAccountServerChanged String
    | AddImapAccountPasswordChanged String
    | AddImapAccountUsernameChanged String
    | AddImapAccountSubmitted


type LogInFormMessage
    = LogInUsernameChanged String
    | LogInPasswordChanged String
    | LogInSubmitted


type NewAccountFormMessage
    = UsernameChanged String
    | EmailChanged String
    | PasswordChanged String
    | NewAccountSubmitted


type Msg
    = LogInClicked
    | NewAccountClicked
    | NewAccountFormMessage NewAccountFormMessage
    | NewAccountRegistered (Result Http.Error String)
    | LogInFormMessage LogInFormMessage
    | LogInValidated (Result Http.Error String)
    | AddImapAccountClicked
    | AddImapAccountFormMessage AddImapAccountFormMessage
    | AddImapAccountValidated (Result Http.Error String)


type Model
    = Portal
    | LogIn LogInFormContent
    | Subscribing Account
    | LoggingIn LogInFormContent
    | Home
    | NewAccount Account
    | AddImapAccount ImapAccount
    | AddingImapAccount ImapAccount


init : () -> ( Model, Cmd Msg )
init _ =
    ( Portal, Cmd.none )



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

        LogInClicked ->
            ( LogIn emptyLogInFormContent, Cmd.none )

        LogInFormMessage formMsg ->
            updateLogInForm formMsg model

        LogInValidated content ->
            ( Home, Cmd.none )

        AddImapAccountClicked ->
            ( AddImapAccount emptyImapAccount, Cmd.none )

        AddImapAccountFormMessage formMsg ->
            updateImapAccountFormMessage formMsg model

        AddImapAccountValidated account ->
            ( Home, Cmd.none )


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
                { url = "/api/new-user"
                , body =
                    Http.stringBody
                        "application/x-www-form-urlencoded"
                        (accountToUrlEncoded currentAccount)
                , expect = Http.expectString NewAccountRegistered
                }
            )

        _ ->
            ( model, Cmd.none )


updateLogInForm : LogInFormMessage -> Model -> ( Model, Cmd Msg )
updateLogInForm msg model =
    case ( msg, model ) of
        ( LogInUsernameChanged newUsername, LogIn content ) ->
            ( LogIn { content | username = newUsername }, Cmd.none )

        ( LogInPasswordChanged newPassword, LogIn content ) ->
            ( LogIn { content | password = newPassword }, Cmd.none )

        ( LogInSubmitted, LogIn content ) ->
            ( LoggingIn content
            , Http.post
                { url = "/api/login"
                , body =
                    Http.stringBody
                        "application/x-www-form-urlencoded"
                        (logInFormContentToUrlEncoded content)
                , expect = Http.expectString LogInValidated
                }
            )

        _ ->
            ( model, Cmd.none )


updateImapAccountFormMessage : AddImapAccountFormMessage -> Model -> ( Model, Cmd Msg )
updateImapAccountFormMessage msg model =
    case ( msg, model ) of
        ( AddImapAccountServerChanged newImapServer, AddImapAccount account ) ->
            ( AddImapAccount { account | imapServer = newImapServer }, Cmd.none )

        ( AddImapAccountUsernameChanged newUsername, AddImapAccount account ) ->
            ( AddImapAccount { account | username = newUsername }, Cmd.none )

        ( AddImapAccountPasswordChanged newPassword, AddImapAccount account ) ->
            ( AddImapAccount { account | password = newPassword }, Cmd.none )

        ( AddImapAccountSubmitted, AddImapAccount account ) ->
            ( AddingImapAccount account
            , Http.post
                { url = "/api/new-imap-account"
                , body =
                    Http.stringBody
                        "application/x-www-form-urlencoded"
                        (addImapServerFormContentToUrlEncoded account)
                , expect = Http.expectString AddImapAccountValidated
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
    [ Background.color colors.background
    , Font.family [ Font.typeface "Roboto Condensed", Font.sansSerif ]
    , Font.size 18
    , Font.color colors.text
    ]


view model =
    case model of
        Portal ->
            portalView

        LogIn content ->
            logInView content

        Home ->
            homeView

        NewAccount account ->
            newAccountView account

        Subscribing account ->
            subscribingView

        LoggingIn account ->
            subscribingView

        AddImapAccount account ->
            imapAccountView account

        AddingImapAccount account ->
            subscribingView


portalView =
    Element.layout defaultAttributes
        myRowOfStuff


homeView =
    newAccountView emptyAccount


subscribingView =
    Element.layout defaultAttributes
        Element.none


newAccountView account =
    Element.layout defaultAttributes
        (loggedInPage account newAccountForm)


logInView content =
    Element.layout defaultAttributes
        (logInForm content)


imapAccountView content =
    Element.layout defaultAttributes
        (imapAccountForm content)


addAccount : Element Msg
addAccount =
    normalButton (Just NewAccountClicked) "New account"


title : String -> Element Msg
title titleText =
    row [ padding 30, centerX, Font.size 32, Font.color colors.accentLight ]
        [ text titleText ]


myRowOfStuff : Element Msg
myRowOfStuff =
    row [ width fill ]
        [ column [ width <| fillPortion 33 ] []
        , column [ width <| fillPortion 33, centerX, alignTop ]
            [ title "Welcome to chouette!"
            , paragraph [ Font.center ]
                [ text "Chouette is a small and reliable open source web mail client written in Rust and Elm."
                , text """It gives you all the keys you need to get started almost instantly, and remain the only
                                person accessing and storing your emails."""
                ]
            , row [ width fill, paddingXY 0 30 ]
                [ el [ width <| fillPortion 50 ] (actionBarButton (Just NewAccountClicked) "New account")
                , el [ width <| fillPortion 50 ] (actionBarButton (Just LogInClicked) "Log in")
                ]
            ]
        , column [ width <| fillPortion 33 ] []
        ]


header : Element Msg
header =
    row
        [ width fill
        , alignTop
        , padding 20
        , Font.color colors.contrastedText
        , Background.gradient { angle = pi / 2, steps = [ colors.accentLight, colors.accent ] }
        ]
        [ column [ width <| fillPortion 3 ]
            [ text "Logo" ]
        , column [ width <| fillPortion 5 ]
            [ text "Search bar" ]
        , column [ width <| fillPortion 5 ]
            [ text "Status bar" ]
        ]


leftMenu : Element Msg
leftMenu =
    column [ width <| fillPortion 25, alignTop ]
        [ menuItem (Just AddImapAccountClicked) "Add new IMAP account"
        ]


menuItem : Maybe Msg -> String -> Element Msg
menuItem message linkText =
    let
        label =
            row
                [ width fill
                , padding 20
                , Background.color colors.accent
                , Font.color colors.contrastedText
                ]
                [ text linkText ]
    in
    Input.button [ width fill, height fill ]
        { onPress = message
        , label = label
        }


loggedInPage : Account -> (Account -> Element Msg) -> Element Msg
loggedInPage account subPage =
    column [ width fill, height fill ]
        [ header
        , row [ width fill, height fill ]
            [ leftMenu
            , column [ width <| fillPortion 75 ] [ subPage account ]
            ]
        ]


normalButton : Maybe Msg -> String -> Element Msg
normalButton onPress buttonText =
    Input.button
        [ Border.rounded 5
        , padding 10
        , Background.color colors.buttonNormal
        , Font.color colors.contrastedText
        , Font.size 12
        ]
        { onPress = onPress
        , label = text buttonText
        }


actionBarButton : Maybe Msg -> String -> Element Msg
actionBarButton onPress buttonText =
    Input.button
        [ padding 10
        , Font.center
        , width fill
        , height fill
        , Background.color colors.buttonNormal
        , Font.color colors.contrastedText
        , Border.color colors.contrastedText
        , Border.widthEach { left = 0, top = 0, right = 1, bottom = 0 }
        ]
        { onPress = onPress
        , label = text (String.toUpper buttonText)
        }


newAccountForm account =
    Element.column [ centerY, centerX ]
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
        , normalButton (Just (NewAccountFormMessage NewAccountSubmitted)) "New account"
        ]


logInForm account =
    Element.column [ centerY, centerX ]
        [ Input.text defaultAttributes
            { onChange = LogInFormMessage << LogInUsernameChanged
            , label = Input.labelAbove (centerY :: defaultAttributes) (text "Username")
            , placeholder = Nothing
            , text = account.username
            }
        , Input.currentPassword defaultAttributes
            { onChange = LogInFormMessage << LogInPasswordChanged
            , label = Input.labelAbove (centerY :: defaultAttributes) (text "Password")
            , placeholder = Nothing
            , text = account.password
            , show = False
            }
        , normalButton (Just (LogInFormMessage LogInSubmitted)) "Log in"
        ]


imapAccountForm account =
    Element.column [ centerY, centerX ]
        [ Input.text defaultAttributes
            { onChange = AddImapAccountFormMessage << AddImapAccountServerChanged
            , label = Input.labelAbove (centerY :: defaultAttributes) (text "Url of the IMAP server")
            , placeholder = Nothing
            , text = account.imapServer
            }
        , Input.text defaultAttributes
            { onChange = AddImapAccountFormMessage << AddImapAccountUsernameChanged
            , label = Input.labelAbove (centerY :: defaultAttributes) (text "Username")
            , placeholder = Nothing
            , text = account.username
            }
        , Input.newPassword defaultAttributes
            { onChange = AddImapAccountFormMessage << AddImapAccountPasswordChanged
            , label = Input.labelAbove (centerY :: defaultAttributes) (text "Password")
            , placeholder = Nothing
            , text = account.password
            , show = False
            }
        , normalButton (Just (AddImapAccountFormMessage AddImapAccountSubmitted)) "Add IMAP account"
        ]
