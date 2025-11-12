import React, { useState, useEffect } from 'react';
import {
  View,
  TouchableOpacity,
  Text,
  Platform,
  PermissionsAndroid,
  Alert,
} from 'react-native';
import MapboxGL from '@rnmapbox/maps';
import Geolocation from '@react-native-community/geolocation';
import { styles } from './styles';

MapboxGL.setAccessToken('pk.eyJ1Ijoib2xlZ3RvdGVtIiwiYSI6ImNtaHV6Nzl4dDA0azQybHNneWlsdThvd3IifQ.2Awquq3TcmtSdir4VhZkeg');

const FIXED_COORDINATES = {
  latitude: 49.8180161,
  longitude: 24.022562,
};

const App = () => {
  const [userLocation, setUserLocation] = useState(null);
  const [targetCoordinate, setTargetCoordinate] = useState(FIXED_COORDINATES);
  const [hasLocationPermission, setHasLocationPermission] = useState(false);
  const [locationError, setLocationError] = useState(null);

  useEffect(() => {
    requestLocationPermission();
  }, []);

  const requestLocationPermission = async () => {
    if (Platform.OS === 'android') {
      try {
        const granted = await PermissionsAndroid.request(
          PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION,
          {
            title: 'Location Permission',
            message: 'This app needs access to your location to show you on the map.',
            buttonPositive: 'OK',
          }
        );
        
        if (granted === PermissionsAndroid.RESULTS.GRANTED) {
          console.log('Location permission granted');
          setHasLocationPermission(true);
          startLocationTracking();
        } else {
          console.log('Location permission denied');
          Alert.alert('Permission Denied', 'Location permission is required to show your position.');
        }
      } catch (err) {
        console.warn('Permission error:', err);
        setLocationError(err.message);
      }
    } else {
      setHasLocationPermission(true);
      startLocationTracking();
    }
  };

  const startLocationTracking = () => {
    console.log('Starting location tracking...');
    
    Geolocation.getCurrentPosition(
      (position) => {
        console.log('Got cached position:', position.coords);
        const newLocation = {
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
        };
        setUserLocation(newLocation);
        setLocationError(null);
      },
      (error) => {
        console.log('No cached location');
      },
      { 
        enableHighAccuracy: false,
        timeout: 5000,
        maximumAge: 300000
      }
    );

    setTimeout(() => {
      Geolocation.getCurrentPosition(
        (position) => {
          console.log('Got accurate position:', position.coords);
          const newLocation = {
            latitude: position.coords.latitude,
            longitude: position.coords.longitude,
          };
          setUserLocation(newLocation);
          setLocationError(null);
        },
        (error) => {
          console.log('Accurate GPS failed, using network location');
          if (!userLocation) {
            setLocationError('Using network location (GPS unavailable)');
          }
        },
        { 
          enableHighAccuracy: true, 
          timeout: 30000,
          maximumAge: 0,
          distanceFilter: 0
        }
      );
    }, 1000);

    const watchId = Geolocation.watchPosition(
      (position) => {
        console.log('Position update:', position.coords);
        setUserLocation({
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
        });
        setLocationError(null);
      },
      (error) => {
        console.log('Watch position error:', error);
      },
      { 
        enableHighAccuracy: false,
        distanceFilter: 10,
        interval: 10000,
        fastestInterval: 5000
      }
    );

    return () => Geolocation.clearWatch(watchId);
  };

  const handleGetCoordinates = () => {
    setTargetCoordinate({ ...FIXED_COORDINATES });
    Alert.alert(
      'Coordinates Loaded',
      `Lat: ${FIXED_COORDINATES.latitude}\nLon: ${FIXED_COORDINATES.longitude}`
    );
  };

  const downloadUkraineMap = async () => {
    const progressListener = (offlineRegion, status) => {
      console.log(`Download progress: ${status.percentage}%`);
    };

    const errorListener = (offlineRegion, error) => {
      console.log('Download error:', error);
    };

    try {
      await MapboxGL.offlineManager.createPack(
        {
          name: 'ukraine-map',
          styleURL: MapboxGL.StyleURL.Street,
          bounds: [[22.13, 44.39], [40.23, 52.38]],
          minZoom: 5,
          maxZoom: 14,
        },
        progressListener,
        errorListener
      );
      Alert.alert('Success', 'Ukraine map downloaded for offline use!');
    } catch (error) {
      Alert.alert('Error', 'Failed to download map: ' + error.message);
    }
  };

  const retryLocation = () => {
    setLocationError(null);
    startLocationTracking();
  };

  return (
    <View style={styles.container}>
      <MapboxGL.MapView
        style={styles.map}
        styleURL={MapboxGL.StyleURL.Street}
      >
        <MapboxGL.Camera
          zoomLevel={14}
          centerCoordinate={[targetCoordinate.longitude, targetCoordinate.latitude]}
          animationDuration={1000}
        />

        <MapboxGL.PointAnnotation
          id="target"
          coordinate={[targetCoordinate.longitude, targetCoordinate.latitude]}
        >
          <View style={styles.targetMarker}>
            <View style={styles.targetDot} />
            <Text style={styles.targetLabel}>Target</Text>
          </View>
        </MapboxGL.PointAnnotation>

        {hasLocationPermission && userLocation && (
          <MapboxGL.PointAnnotation
            id="userLocation"
            coordinate={[userLocation.longitude, userLocation.latitude]}
          >
            <View style={styles.userMarker}>
              <View style={styles.userDot} />
              <Text style={styles.userLabel}>You</Text>
            </View>
          </MapboxGL.PointAnnotation>
        )}
      </MapboxGL.MapView>

      <TouchableOpacity style={styles.button} onPress={handleGetCoordinates}>
        <Text style={styles.buttonText}>Get Coordinates</Text>
      </TouchableOpacity>

      {/* <TouchableOpacity style={styles.downloadButton} onPress={downloadUkraineMap}> */}
      <TouchableOpacity style={styles.downloadButton} disabled={true}>
        <Text style={styles.buttonText}>Download Ukraine Map</Text>
      </TouchableOpacity>

      <View style={styles.infoBox}>
        <Text style={styles.infoText}>
          Target: {targetCoordinate.latitude.toFixed(6)}, {targetCoordinate.longitude.toFixed(6)}
        </Text>
        {userLocation && (
          <Text style={styles.infoText}>
            You: {userLocation.latitude.toFixed(6)}, {userLocation.longitude.toFixed(6)}
          </Text>
        )}
        {!userLocation && hasLocationPermission && (
          <Text style={styles.infoText}>Getting your location...</Text>
        )}
        {locationError && (
          <>
            <Text style={[styles.infoText, {color: 'red'}]}>Error: {locationError}</Text>
            <TouchableOpacity onPress={retryLocation}>
              <Text style={[styles.infoText, {color: '#007AFF', fontWeight: 'bold'}]}>Tap to retry</Text>
            </TouchableOpacity>
          </>
        )}
      </View>
    </View>
  );
};

export default App;