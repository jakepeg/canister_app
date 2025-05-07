Redesign Specification: Canister Selection Refactor
1. Overview

This specification details the redesign required to refactor the application's main page. Currently, the main page displays a list of files directly. The refactor will change this so the main page first presents a list of available canisters. Selecting a canister will then navigate the user to the file list specific to that canister. This change introduces a new canister selection screen and a modal for creating new canisters, based on the provided Figma designs.

2. Goals

Replace the current file list on the main route (/) with a canister selection screen.
Implement the canister selection screen UI based on the "Canister List" and "Canister List empty" Figma designs.
Implement a modal for creating new canisters based on the "Canister list popup" Figma design.
Ensure selecting a canister navigates the user to its corresponding file list view (the existing functionality, but triggered after canister selection).
3. User Flow

User logs in and lands on the main page (/).
The page displays a list of the user's canisters OR an empty state if they have none.
If canisters exist:
User clicks on a canister card.
User is navigated to the file list view for that specific canister.
If no canisters exist (Empty State):
The page displays a message prompting the user to create a canister.
User clicks the "New Canister" button.
Creating a Canister:
User clicks the "New Canister" button (available in both list and empty states).
The "Create New Canister" modal appears.
User enters a name for the canister.
User clicks the "Create Canister" button within the modal.
The modal closes, the canister list refreshes (or transitions from empty state), showing the newly created canister.
4. Component Redesign & Implementation Details

Based on the Figma nodes 288:76 (Canister List), 280:6 (Canister List empty), and 289:124 (Canister list popup):

4.1. Main Page (frontend/src/frontend/src/routes/+page.svelte)

Refactor: Remove the existing file list logic.
Implement:
Fetch the list of canisters associated with the logged-in user upon page load.
Conditionally render either the CanisterList component (passing the fetched canisters as a prop) or the empty state view if no canisters are returned.
Integrate the CreateCanisterModal component, likely triggered by an event dispatched from the CanisterList component.
4.2. Canister List Component (frontend/src/frontend/src/lib/components/Canisters/CanisterList.svelte - To be created or heavily modified)

Purpose: Displays the list of canisters or the empty state message.
Props: canisters: CanisterInfo[] (Define CanisterInfo type based on backend data, likely including id and name).
UI (Based on Figma 288:76 & 280:6):
Layout: A grid or flex container to display canister cards.
Header: Display the text "My Canisters" (Style: style_ESKRTZ - Inder, 20px, White).
"New Canister" Button:
Button with text "New Canister" (Style: style_GUBF0I - Inder, 17px, White).
Styled with a white stroke, 6px border-radius (Based on 280:9, 288:81).
On click: Dispatch an event to the parent (+page.svelte) to open the Create Canister Modal.
Canister Cards (If canisters prop is not empty):
Iterate over the canisters prop.
Each card should be a clickable element that navigates the user to the file list for that canister (e.g., /canister/{canisterId}/files).
Card Styling (Based on 288:110, 288:115): Dark background (#1F1F1F inferred), white stroke (stroke_XW5FIK), 15px border-radius, box-shadow (effect_8Z75BV).
Card Content:
Display canister image/icon (Placeholder DALL_E... image fill_U5YNZA in Figma). A default icon can be used initially.
Display canister name (Text Style: style_LZI9YD - Inder, 16px, White, Centered).
Include three dots icon (Based on 288:119, 288:120) for potential future actions (delete, rename). Initially, this can be non-functional or omitted.
Empty State (If canisters prop is empty):
Display the logo/icon (Based on 280:30).
Display the text "Create a canister to get started." (Style: style_8GQ93Y - Inder, 20px, White, Centered).
4.3. Create Canister Modal Component (frontend/src/frontend/src/lib/components/Canisters/CreateCanisterModal.svelte - New Component)

Purpose: Provides a modal dialog for creating a new canister.
UI (Based on Figma 289:141):
Backdrop: Semi-transparent overlay (Based on Rectangle 107 fill_RMREMI).
Modal Container:
Background: Dark (#1F1F1F inferred).
Border: 2px stroke, color #0B8CE9 (stroke_ASSQ7O).
Border Radius: 21px.
Header:
Title: "Create New Canister" (Style: style_GUBF0I - Inder, 17px, White).
Close Button: "X" icon/button (Style: style_GUBF0I - Inder, 17px, White). On click, closes the modal.
Form Fields:
Canister Name Input:
Label: "Canister Name" (Style: style_4O2OYN - Inder, 15px, White).
Input field styling: Based on Rectangle 93 (289:150) - 1px stroke (#0B8CE9), 9px border-radius. Placeholder text style: style_RLWTAB with fill_XAIZZZ (Inder, 16px, White 52% opacity).
Size Display:
Label: "Size (GB)" (Style: style_4O2OYN - Inder, 15px, White).
Display field styling: Based on Rectangle 94 (289:151) - 1px stroke (#0B8CE9), 9px border-radius.
Value: Display "500gb" statically for now (Style: style_RLWTAB with fill_XAIZZZ). Note: Confirm if size is configurable or fixed.
Setup Cost Display:
Label: "Setup Cost:" (Style: style_GUBF0I - Inder, 17px, White).
Value: Needs clarification from backend/requirements if this is dynamic or static. Display appropriately.
Action Button:
Button text: "Create Canister" (Style: style_GUBF0I - Inder, 17px, White).
Button styling: Based on Rectangle 95 (289:154) - White stroke (#FFFFFF), 22px border-radius.
On click: Trigger the backend canister creation logic, handle loading/success/error states, and close the modal on success, potentially dispatching an event to refresh the canister list.
5. Styling Notes

Colors: Primarily dark backgrounds (#1F1F1F, #020817), white text, and blue accents (#0B8CE9).
Fonts: Use "Inder" font as specified in the styles (style_ESKRTZ, style_GUBF0I, etc.).
Spacing/Padding: Infer from Figma layout where possible, maintain consistency.
Effects: Apply box-shadows as defined in Figma (effect_8Z75BV).
6. Assets

Canister icon/image (Placeholder used in Figma, a default asset might be needed).
Logo icon for empty state (Based on 280:30).
Three dots icon (if implemented).
7. Open Questions/Assumptions

Is the canister size fixed at 500gb, or should it be configurable? The design shows it statically.
How is the "Setup Cost" determined and displayed?
What is the exact navigation path for a selected canister (e.g., /canister/{id}/files)?
What specific backend calls are needed for fetching canisters and creating a new one?
Error handling specifics for canister creation failure.
This specification provides a detailed plan based on the Figma designs. 
