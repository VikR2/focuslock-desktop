import FavoritesBar from "../FavoritesBar";

export default function FavoritesBarExample() {
  const handleAddFavorite = () => {
    console.log("Add favorite clicked");
  };

  return (
    <div className="h-96">
      <FavoritesBar
        onAddFavorite={handleAddFavorite}
      />
    </div>
  );
}