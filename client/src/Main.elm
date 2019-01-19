module Main exposing (main)

import Browser
import Either exposing (Either)
import Element exposing (Element)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Html
import Http
import Styles exposing (colors, defaultAttributes)


main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }



-------------------------------------------------------------------------------
-- TYPES ----------------------------------------------------------------------
-------------------------------------------------------------------------------
-- HELPER TYPES ---------------------------------------------------------------


type FormStatus
    = Idle
    | Submitted
    | Success
    | Failure



-- LOG IN FORM ----------------------------------------------------------------


type alias LogInFormContent =
    { username : String
    , password : String
    , status : FormStatus
    }


logInFormContentToUrlEncoded : LogInFormContent -> String
logInFormContentToUrlEncoded content =
    String.join "&"
        [ "username=" ++ content.username
        , "password=" ++ content.password
        ]


defaultLogInFormContent : LogInFormContent
defaultLogInFormContent =
    { username = "", password = "", status = Idle }


type LogInFormMsg
    = LogInFormUsernameChanged String
    | LogInFormPasswordChanged String
    | LogInFormSubmitted
    | LogInFormResponse (Result Http.Error String)



-- REGISTER FORM --------------------------------------------------------------


type alias RegisterFormContent =
    { username : String
    , email : String
    , password : String
    , status : FormStatus
    }


registerFormContentToUrlEncoded : RegisterFormContent -> String
registerFormContentToUrlEncoded content =
    String.join "&"
        [ "username=" ++ content.username
        , "email=" ++ content.email
        , "password=" ++ content.password
        ]


defaultRegisterFormContent : RegisterFormContent
defaultRegisterFormContent =
    { username = ""
    , email = ""
    , password = ""
    , status = Idle
    }


type RegisterFormMsg
    = RegisterFormUsernameChanged String
    | RegisterFormEmailChanged String
    | RegisterFormPasswordChanged String
    | RegisterFormSubmitted
    | RegisterFormResponse (Result Http.Error String)



-- MAIN -----------------------------------------------------------------------


type PortalForm
    = LogInForm
    | RegisterForm


type alias PortalContent =
    { logInForm : LogInFormContent
    , registerForm : RegisterFormContent
    , form : PortalForm
    }


type Model
    = Portal PortalContent
    | Home String


type Msg
    = LogInFormMsg LogInFormMsg
    | RegisterFormMsg RegisterFormMsg
    | GoToLogInFormMsg
    | GoToRegisterFormMsg
    | MailboxesMsg (Result Http.Error String)


defaultPortalContent : PortalContent
defaultPortalContent =
    { logInForm = defaultLogInFormContent
    , registerForm = defaultRegisterFormContent
    , form = LogInForm
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Portal defaultPortalContent, Cmd.none )



-------------------------------------------------------------------------------
-- UPDATES --------------------------------------------------------------------
-------------------------------------------------------------------------------


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model ) of
        ( LogInFormMsg message, Portal content ) ->
            let
                ( result, cmd ) =
                    updateLogInForm message content.logInForm
            in
            case result of
                Either.Left newLogInForm ->
                    ( Portal { content | logInForm = newLogInForm }, cmd )

                Either.Right m ->
                    ( m, cmd )

        ( RegisterFormMsg message, Portal content ) ->
            let
                ( newRegisterForm, cmd ) =
                    updateRegisterForm message content.registerForm
            in
            ( Portal { content | registerForm = newRegisterForm }, cmd )

        ( GoToLogInFormMsg, Portal content ) ->
            ( Portal { content | form = LogInForm }, Cmd.none )

        ( GoToRegisterFormMsg, Portal content ) ->
            ( Portal { content | form = RegisterForm }, Cmd.none )

        ( MailboxesMsg (Ok mailboxes), _ ) ->
            ( Home mailboxes, Cmd.none )

        ( _, m ) ->
            ( m, Cmd.none )


{-|

    This function can either return a delta on the log in form content, or a
    completely new model in case the log in is succesful.

-}
updateLogInForm : LogInFormMsg -> LogInFormContent -> ( Either LogInFormContent Model, Cmd Msg )
updateLogInForm msg logInForm =
    case msg of
        LogInFormUsernameChanged newUsername ->
            ( Either.Left { logInForm | username = newUsername }, Cmd.none )

        LogInFormPasswordChanged newPassword ->
            ( Either.Left { logInForm | password = newPassword }, Cmd.none )

        LogInFormSubmitted ->
            ( Either.Left { logInForm | status = Submitted }, requestLogIn logInForm )

        LogInFormResponse (Ok resonse) ->
            ( Either.Left { logInForm | status = Success }, requestMailboxes )

        LogInFormResponse (Err resonse) ->
            ( Either.Left { logInForm | status = Failure }, Cmd.none )


updateRegisterForm : RegisterFormMsg -> RegisterFormContent -> ( RegisterFormContent, Cmd Msg )
updateRegisterForm msg registerForm =
    case msg of
        RegisterFormUsernameChanged newUsername ->
            ( { registerForm | username = newUsername }, Cmd.none )

        RegisterFormPasswordChanged newPassword ->
            ( { registerForm | password = newPassword }, Cmd.none )

        RegisterFormEmailChanged newEmail ->
            ( { registerForm | email = newEmail }, Cmd.none )

        RegisterFormSubmitted ->
            ( { registerForm | status = Submitted }, requestRegister registerForm )

        RegisterFormResponse (Ok response) ->
            ( { registerForm | status = Success }, Cmd.none )

        RegisterFormResponse (Err response) ->
            ( { registerForm | status = Failure }, Cmd.none )



-------------------------------------------------------------------------------
-- COMMANDS -------------------------------------------------------------------
-------------------------------------------------------------------------------


httpStringBody : String -> Http.Body
httpStringBody params =
    Http.stringBody "application/x-www-form-urlencoded" params


requestLogIn : LogInFormContent -> Cmd Msg
requestLogIn content =
    Http.post
        { url = "/api/login"
        , body = httpStringBody (logInFormContentToUrlEncoded content)
        , expect = Http.expectString (LogInFormMsg << LogInFormResponse)
        }


requestRegister : RegisterFormContent -> Cmd Msg
requestRegister content =
    Http.post
        { url = "/api/new-user"
        , body = httpStringBody (registerFormContentToUrlEncoded content)
        , expect = Http.expectString (RegisterFormMsg << RegisterFormResponse)
        }


requestMailboxes : Cmd Msg
requestMailboxes =
    Http.post
        { url = "/api/get-mailboxes"
        , body = Http.emptyBody
        , expect = Http.expectString MailboxesMsg
        }



-------------------------------------------------------------------------------
-- SUBSCRIPTIONS --------------------------------------------------------------
-------------------------------------------------------------------------------


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-------------------------------------------------------------------------------
-- VIEWS ----------------------------------------------------------------------
-------------------------------------------------------------------------------


view : Model -> Html.Html Msg
view model =
    case model of
        Portal content ->
            portalView content

        Home mailboxes ->
            homeView mailboxes



-- HOME VIEWS -----------------------------------------------------------------


homeView : String -> Html.Html Msg
homeView mailboxes =
    Element.layout defaultAttributes
        (Element.column
            (Element.width Element.fill :: defaultAttributes)
            [ header
            , Element.row
                (Element.height Element.fill :: defaultAttributes)
                [ leftMenu
                , Element.column defaultAttributes
                    [ Element.text mailboxes
                    ]
                ]
            ]
        )


leftMenu : Element Msg
leftMenu =
    Element.column [ Element.width <| Element.fillPortion 25, Element.alignTop ]
        [ menuItem Nothing "Add new IMAP account"
        ]


menuItem : Maybe Msg -> String -> Element Msg
menuItem message linkText =
    let
        label =
            Element.row
                [ Element.width Element.fill
                , Element.padding 20
                , Background.color colors.accent
                , Font.color colors.contrastedText
                ]
                [ Element.text linkText ]
    in
    Input.button [ Element.width Element.fill, Element.height Element.fill ]
        { onPress = message
        , label = label
        }



-- PORTAL VIEWS ---------------------------------------------------------------


portalView : PortalContent -> Html.Html Msg
portalView content =
    Element.layout defaultAttributes
        (Element.column
            (Element.width Element.fill :: defaultAttributes)
            [ header
            , portalContent content
            ]
        )


portalContent : PortalContent -> Element Msg
portalContent content =
    Element.row (Element.width Element.fill :: defaultAttributes)
        [ emptyColumn 1
        , portalPresentation
        , emptyColumn 2
        , portalForm content
        , emptyColumn 1
        ]


portalForm : PortalContent -> Element Msg
portalForm content =
    case content.form of
        LogInForm ->
            portalLogInForm content.logInForm

        RegisterForm ->
            portalRegisterForm content.registerForm


portalLogInForm : LogInFormContent -> Element Msg
portalLogInForm content =
    let
        text =
            case content.status of
                Idle ->
                    Element.text "Log in"

                Submitted ->
                    Element.text "Logging in..."

                Success ->
                    Element.text "Logged in!"

                Failure ->
                    Element.text "An error occured :'( Click to retry"

        message =
            if content.status == Idle || content.status == Failure then
                Just (LogInFormMsg LogInFormSubmitted)

            else
                Nothing
    in
    Element.column [ Element.centerY, Element.spacing 10, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Log in"
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Username")
            , onChange = LogInFormMsg << LogInFormUsernameChanged
            , placeholder = Nothing
            , text = content.username
            }
        , Input.currentPassword Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Password")
            , onChange = LogInFormMsg << LogInFormPasswordChanged
            , placeholder = Nothing
            , text = content.password
            , show = False
            }
        , Input.button
            (Element.centerX
                :: Border.solid
                :: Border.width 1
                :: Border.rounded 5
                :: Background.color Styles.colors.buttonNormal
                :: Element.padding 10
                :: Styles.defaultAttributes
            )
            { label = text
            , onPress = message
            }
        , Input.button Styles.defaultAttributes
            { label = Element.text "Not registered yet ? Click here to register."
            , onPress = Just GoToRegisterFormMsg
            }
        ]


portalRegisterForm : RegisterFormContent -> Element Msg
portalRegisterForm content =
    let
        text =
            case content.status of
                Idle ->
                    Element.text "Register"

                Submitted ->
                    Element.text "Registering..."

                Success ->
                    Element.text "Succesfully registered!"

                Failure ->
                    Element.text "An error occured :'("

        message =
            if content.status == Idle then
                Just (RegisterFormMsg RegisterFormSubmitted)

            else
                Nothing
    in
    Element.column [ Element.centerY, Element.spacing 10, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Log in"
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Username")
            , onChange = RegisterFormMsg << RegisterFormUsernameChanged
            , placeholder = Nothing
            , text = content.username
            }
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Email")
            , onChange = RegisterFormMsg << RegisterFormEmailChanged
            , placeholder = Nothing
            , text = content.email
            }
        , Input.currentPassword Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Password")
            , onChange = RegisterFormMsg << RegisterFormPasswordChanged
            , placeholder = Nothing
            , text = content.password
            , show = False
            }
        , Input.button
            (Element.centerX
                :: Border.solid
                :: Border.width 1
                :: Border.rounded 5
                :: Background.color Styles.colors.buttonNormal
                :: Element.padding 10
                :: Styles.defaultAttributes
            )
            { label = text
            , onPress = message
            }
        , Input.button Styles.defaultAttributes
            { label = Element.text "Already registered ? Click here to log in."
            , onPress = Just GoToLogInFormMsg
            }
        ]


portalPresentation : Element msg
portalPresentation =
    Element.column [ Element.centerY, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Welcome to chouette!"
        , Element.paragraph [ Font.center ]
            [ Element.text description ]
        ]



-- UTILS ----------------------------------------------------------------------


emptyColumn : Int -> Element Msg
emptyColumn portion =
    Element.column [ Element.width <| Element.fillPortion portion ] []


header : Element msg
header =
    Element.row
        [ Element.width Element.fill
        , Element.alignTop
        , Element.padding 20
        , Font.color colors.contrastedText
        , Background.gradient { angle = pi / 2, steps = [ colors.accentLight, colors.accent ] }
        ]
        [ Element.column [ Element.width <| Element.fillPortion 3 ]
            [ Element.text "Logo" ]
        , Element.column [ Element.width <| Element.fillPortion 5 ]
            [ Element.text "Search bar" ]
        , Element.column [ Element.width <| Element.fillPortion 5 ]
            [ Element.text "Status bar" ]
        ]


description : String
description =
    """Chouette is a small and reliable open source web mail client written in Rust and Elm. It gives you all the keys you need to get started almost instantly, and remain the only person accessing and storing your emails."""
