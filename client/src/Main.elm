module Main exposing (main)

import Browser
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
    | Received



-- LOG IN FORM ----------------------------------------------------------------


type alias LogInFormContent =
    { username : String
    , password : String
    , submitted : Bool
    }


logInFormContentToUrlEncoded : LogInFormContent -> String
logInFormContentToUrlEncoded content =
    String.join "&"
        [ "username=" ++ content.username
        , "password=" ++ content.password
        ]


defaultLogInFormContent : LogInFormContent
defaultLogInFormContent =
    { username = "", password = "", submitted = False }


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


type Msg
    = LogInFormMsg LogInFormMsg
    | RegisterFormMsg RegisterFormMsg
    | GoToLogInForm
    | GoToRegisterForm


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
                ( newLogInForm, cmd ) =
                    updateLogInForm message content.logInForm
            in
            ( Portal { content | logInForm = newLogInForm }, cmd )

        ( RegisterFormMsg message, Portal content ) ->
            let
                ( newRegisterForm, cmd ) =
                    updateRegisterForm message content.registerForm
            in
            ( Portal { content | registerForm = newRegisterForm }, cmd )

        ( GoToLogInForm, Portal content ) ->
            ( Portal { content | form = LogInForm }, Cmd.none )

        ( GoToRegisterForm, Portal content ) ->
            ( Portal { content | form = RegisterForm }, Cmd.none )


updateLogInForm : LogInFormMsg -> LogInFormContent -> ( LogInFormContent, Cmd Msg )
updateLogInForm msg logInForm =
    case msg of
        LogInFormUsernameChanged newUsername ->
            ( { logInForm | username = newUsername }, Cmd.none )

        LogInFormPasswordChanged newPassword ->
            ( { logInForm | password = newPassword }, Cmd.none )

        LogInFormSubmitted ->
            ( { logInForm | submitted = True }, sendLogInRequest logInForm )

        LogInFormResponse resonse ->
            ( { logInForm | submitted = False }, Cmd.none )


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
            ( { registerForm | status = Submitted }, sendRegisterRequest registerForm )

        RegisterFormResponse resonse ->
            ( { registerForm | status = Received }, Cmd.none )



-------------------------------------------------------------------------------
-- COMMANDS -------------------------------------------------------------------
-------------------------------------------------------------------------------


httpStringBody : String -> Http.Body
httpStringBody params =
    Http.stringBody "application/x-www-form-urlencoded" params


sendLogInRequest : LogInFormContent -> Cmd Msg
sendLogInRequest content =
    Http.post
        { url = "/api/login"
        , body = httpStringBody (logInFormContentToUrlEncoded content)
        , expect = Http.expectString (LogInFormMsg << LogInFormResponse)
        }


sendRegisterRequest : RegisterFormContent -> Cmd Msg
sendRegisterRequest content =
    Http.post
        { url = "/api/new-user"
        , body = httpStringBody (registerFormContentToUrlEncoded content)
        , expect = Http.expectString (LogInFormMsg << LogInFormResponse)
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
            if content.submitted then
                Element.text "Logging in..."

            else
                Element.text "Log in"

        message =
            if content.submitted then
                Nothing

            else
                Just (LogInFormMsg LogInFormSubmitted)
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
            , onPress = Just GoToRegisterForm
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

                Received ->
                    Element.text "Succesfully registered!"

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
            , onPress = Just GoToLogInForm
            }
        ]


emptyColumn : Int -> Element Msg
emptyColumn portion =
    Element.column [ Element.width <| Element.fillPortion portion ] []


portalPresentation : Element msg
portalPresentation =
    Element.column [ Element.centerY, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Welcome to chouette!"
        , Element.paragraph [ Font.center ]
            [ Element.text description ]
        ]


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
